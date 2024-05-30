use crate::cli::Arguments;
use crate::config::{TwmGlobal, TwmLocal};
use crate::layout::{get_commands_from_layout, get_commands_from_layout_name, get_layout_names};
use crate::ui::picker::{Picker, PickerSelection};
use anyhow::{bail, Context, Result};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Output};

pub struct SessionName {
    name: String,
}

impl SessionName {
    pub fn new(path: &str, path_components: usize) -> Self {
        let mut path_parts: Vec<&str> = path.split('/').rev().take(path_components).collect();
        path_parts.reverse();
        let raw_name = path_parts.join("/");
        Self::from(raw_name.as_str())
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl From<&str> for SessionName {
    // take the last 2 parts of the path and join them back together, replacing any illegal characters with an underscore
    fn from(s: &str) -> Self {
        let name: String = s
            .chars()
            .map(|c| match c {
                // TODO: go through and find where tmux does the char replacement to get a full list of illegal characters. is it just this?
                '.' => '_',
                _ => c,
            })
            .collect();
        SessionName { name }
    }
}
fn run_tmux_command(args: &[&str]) -> Result<Output> {
    let output = Command::new("tmux")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run tmux command with args {args:?}"))?;
    if !output.status.success() {
        bail!(
            "tmux command with args {:?} failed because: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output)
}

pub fn get_tmux_sessions() -> Result<Vec<String>> {
    let output = run_tmux_command(&["list-sessions", "-F", "#{session_name}"])?;
    let out_str = String::from_utf8_lossy(&output.stdout);
    let sessions: Vec<String> = out_str.lines().map(|s| s.to_string()).collect();
    Ok(sessions)
}

fn create_tmux_session(name: &SessionName, workspace_type: Option<&str>, path: &str) -> Result<()> {
    run_tmux_command(&[
        "new-session",
        "-ds",
        &name.name,
        "-t",
        &name.name,
        "-c",
        path,
        // set TWM env vars for the session
        "-e",
        "TWM=1",
        "-e",
        &format!("TWM_ROOT={}", path),
        "-e",
        &format!("TWM_TYPE={}", workspace_type.unwrap_or("")),
        "-e",
        &format!("TWM_NAME={}", name.name),
    ])
    .with_context(|| {
        format!(
            "Failed to create tmux session with name {} at path {path}",
            &name.name
        )
    })?;
    Ok(())
}

fn create_tmux_session_in_group(group_session_name: &str, name: &SessionName) -> Result<()> {
    run_tmux_command(&["new-session", "-ds", &name.name, "-t", group_session_name]).with_context(
        || {
            format!(
                "Failed to create tmux session {} in group {}",
                &name.name, group_session_name
            )
        },
    )?;
    Ok(())
}

fn attach_to_tmux_session_inside_tmux(session_name: &str) -> Result<()> {
    run_tmux_command(&["switch", "-t", session_name]).with_context(|| {
        format!("Failed to attach to tmux session with name {session_name} inside tmux")
    })?;
    Ok(())
}

pub fn attach_to_tmux_session(session_name: &str) -> Result<()> {
    if std::env::var("TMUX").is_ok() {
        attach_to_tmux_session_inside_tmux(session_name)
    } else {
        attach_to_tmux_session_outside_tmux(session_name)
    }
}

fn attach_to_tmux_session_outside_tmux(session_name: &str) -> Result<()> {
    let shell = std::env::var("SHELL").unwrap_or("sh".to_string());
    let exec_error = Command::new(shell)
        .args(["-c", format!("tmux attach -t {}", session_name).as_str()])
        .exec();
    anyhow::bail!(
        "Failed to attach to tmux session with name {repo_name} outside tmux: {exec_error}",
        repo_name = session_name,
        exec_error = exec_error
    );
}

fn tmux_has_session(session_name: &SessionName) -> bool {
    match run_tmux_command(&["has-session", "-t", &session_name.name]) {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn get_twm_root_for_session(session_name: &SessionName) -> Result<String> {
    let output = run_tmux_command(&["showenv", "-t", &session_name.name])?;
    let out_str = String::from_utf8_lossy(&output.stdout);
    let twm_root = out_str
        .lines()
        .find(|line| line.starts_with("TWM_ROOT="))
        .with_context(|| {
            format!(
                "Failed to find TWM_ROOT variable in tmux session {}",
                session_name.name
            )
        })?
        .strip_prefix("TWM_ROOT=")
        .with_context(|| {
            format!(
                "Failed to strip TWM_ROOT= prefix from tmux session {}",
                session_name.name
            )
        })?
        .to_string();

    Ok(twm_root)
}

fn send_commands_to_session(session_name: &str, commands: &[&str]) -> Result<()> {
    for command in commands {
        run_tmux_command(&["send-keys", "-t", session_name, command, "C-m"])?;
    }
    Ok(())
}

fn get_layout_selection(twm_config: &TwmGlobal) -> Result<String> {
    Ok(
        match Picker::new(
            &get_layout_names(&twm_config.layouts),
            "Select a layout: ".into(),
        )
        .get_selection()?
        {
            PickerSelection::None => bail!("No layout selected"),
            PickerSelection::Selection(s) => s,
            PickerSelection::ModifiedSelection(s) => s,
        },
    )
}

fn get_workspace_commands<'a>(
    workspace_type: Option<&str>,
    twm_config: &'a TwmGlobal,
    cli_layout: Option<&'a str>,

    local_config: Option<&'a TwmLocal>,
) -> Result<Option<Vec<&'a str>>> {
    // if user wants to choose a layout do this first
    if let Some(cli_layout) = cli_layout {
        return Ok(Some(get_commands_from_layout_name(
            cli_layout,
            &twm_config.layouts,
        )));
    }

    // next check if a local layout exists
    if let Some(local) = local_config {
        return Ok(Some(get_commands_from_layout(
            &local.layout,
            &twm_config.layouts,
        )));
    }

    match workspace_type {
        Some(t) => {
            for workspace_definition in &twm_config.workspace_definitions {
                if workspace_definition.name == t {
                    if let Some(layout_name) = &workspace_definition.default_layout {
                        return Ok(Some(get_commands_from_layout_name(
                            layout_name,
                            &twm_config.layouts,
                        )));
                    } else {
                        return Ok(None);
                    }
                }
            }
            Ok(None)
        }
        None => Ok(None),
    }
}

fn find_config_file(workspace_path: &Path) -> Result<Option<TwmLocal>> {
    let local_config = TwmLocal::load(workspace_path)?;
    if let Some(local_config) = local_config {
        return Ok(Some(local_config));
    }
    match workspace_path.parent() {
        Some(parent) => find_config_file(parent),
        None => Ok(None),
    }
}

pub fn session_name_for_path_recursive(
    path: &str,
    path_components: usize,
) -> Result<Option<SessionName>> {
    // start out with the session name for the base # of path components passed in
    let name = SessionName::new(path, path_components);

    // if no session with the auto-generated name exists, we say there is no session
    // technically this won't work for custom-named sessions, but the original intention behind
    // allowing a custom name was to keep those sessions somewhat isolated from the builtin functionalities
    // so for now i am calling that behavior a feature not a bug
    if !tmux_has_session(&name) {
        return Ok(None);
    }

    // if we successfully parse the TWM_ROOT variable for the session and it matches our path,
    // we've found the session we're looking for & return that session name
    if let Ok(twm_root) = get_twm_root_for_session(&name) {
        if twm_root == path {
            return Ok(Some(name));
        }
    }
    // if we have an error or our path doesn't match the TWM_ROOT, add more path components
    session_name_for_path_recursive(path, path_components + 1)
}

fn get_session_name_recursive(path: &str, path_components: usize) -> Result<SessionName> {
    let name = SessionName::new(path, path_components);
    // no session means we can use this name
    if !tmux_has_session(&name) {
        return Ok(name);
    }

    // if the name already exists, there are two cases:
    // 1. the session is a twm session, in which case we can extract the TWM_ROOT env var to check if it matches the current path
    // 2. the session is not a twm session, in which case we need to recurse and try a new name
    match get_twm_root_for_session(&name) {
        // if we successfully get the TWM_ROOT variable, we are in a TWM session. if TWM_ROOT matches the path we're currently trying
        // to open, we can use this name and will simply attach to the existing session
        Ok(twm_root) => {
            if twm_root == path {
                Ok(name)
            } else {
                // if TWM_ROOT doesn't match, we've had a name collision and need to recurse and try a new name with more path components
                let new_name = get_session_name_recursive(path, path_components + 1)?;
                Ok(new_name)
            }
        }
        // if we fail to get the TWM_ROOT variable, either the session is not a TWM session or is broken (e.g. TWM_ROOT is not set)
        // either way we still need to recurse for a new name
        Err(_) => {
            let new_name = get_session_name_recursive(path, path_components + 2)?;
            Ok(new_name)
        }
    }
}

fn get_group_session_name(group_session_name: &str) -> Result<SessionName> {
    let mut name_iter = 1;
    let mut temp_name = format!("{}-{}", group_session_name, name_iter);
    let mut name = SessionName::from(temp_name.as_str());
    while tmux_has_session(&name) {
        name_iter += 1;
        temp_name = format!("{}-{}", group_session_name, name_iter);
        name = SessionName::from(temp_name.as_str());
    }
    Ok(name)
}

pub fn open_workspace(
    workspace_path: &str,
    workspace_type: Option<&str>,
    config: &TwmGlobal,
    args: &Arguments,
) -> Result<()> {
    let tmux_name = match &args.name {
        Some(name) => SessionName::from(name.as_str()),
        None => get_session_name_recursive(workspace_path, config.session_name_path_components)?,
    };
    if !tmux_has_session(&tmux_name) {
        create_tmux_session(&tmux_name, workspace_type, workspace_path)?;
        let local_config = find_config_file(Path::new(workspace_path))?;
        let cli_layout = if args.layout {
            Some(get_layout_selection(config)?)
        } else {
            None
        };
        let commands = get_workspace_commands(
            workspace_type,
            config,
            cli_layout.as_deref(),
            local_config.as_ref(),
        )?;
        if let Some(layout_commands) = commands {
            send_commands_to_session(&tmux_name.name, &layout_commands)?;
        }
    }
    if !args.dont_attach {
        attach_to_tmux_session(&tmux_name.name)?;
    }
    Ok(())
}

pub fn open_workspace_in_group(group_session_name: &str, args: &Arguments) -> Result<()> {
    let tmux_name = match &args.name {
        Some(name) => SessionName::from(name.as_str()),
        None => get_group_session_name(group_session_name)?,
    };
    create_tmux_session_in_group(group_session_name, &tmux_name)?;
    if !args.dont_attach {
        attach_to_tmux_session(&tmux_name.name)?;
    }

    Ok(())
}
