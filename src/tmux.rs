use crate::cli::Arguments;
use crate::config::{LayoutDefinition, TwmGlobal, TwmLocal};
use crate::matches::SafePath;
use crate::picker::get_skim_selection_from_slice;
use anyhow::{bail, Context, Result};
use libc::execvp;
use std::ffi::CString;
use std::path::Path;
use std::process::Command;

pub struct SessionName {
    name: String,
}

impl From<&str> for SessionName {
    // take the last 2 parts of the path and join them back together, replacing any illegal characters with an underscore
    fn from(s: &str) -> Self {
        let mut last_two_parts: Vec<&str> = s.split('/').rev().take(2).collect();
        last_two_parts.reverse();

        let mut name = last_two_parts.join("/");

        // i know theres more but ill add them when i run into them again
        let disallowed_chars = vec!["."];
        for disallowed_char in disallowed_chars {
            name = name.replace(disallowed_char, "_");
        }
        SessionName { name }
    }
}

fn run_tmux_command(args: &[&str]) -> Result<()> {
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
    Ok(())
}

fn create_tmux_session(name: &SessionName, path: &str) -> Result<()> {
    run_tmux_command(&["new-session", "-ds", &name.name, "-c", path]).with_context(|| {
        format!(
            "Failed to create tmux session with name {} at path {path}",
            &name.name
        )
    })?;
    Ok(())
}

fn attach_to_tmux_session_inside_tmux(session_name: &str) -> Result<()> {
    run_tmux_command(&["switch", "-t", session_name]).with_context(|| {
        format!("Failed to attach to tmux session with name {session_name} inside tmux")
    })?;
    Ok(())
}

fn attach_to_tmux_session(session_name: &str) -> Result<()> {
    if std::env::var("TMUX").is_ok() {
        attach_to_tmux_session_inside_tmux(session_name)
    } else {
        attach_to_tmux_session_outside_tmux(session_name)
    }
}

fn attach_to_tmux_session_outside_tmux(repo_name: &str) -> Result<()> {
    let tmux_attach = CString::new("tmux").unwrap();
    let tmux_attach_args = vec![
        CString::new("tmux").unwrap(),
        CString::new("attach").unwrap(),
        CString::new("-t").unwrap(),
        CString::new(repo_name).with_context(|| "Unable to turn repo name to a cstring.")?,
    ];

    let tmux_attach_args_ptrs: Vec<*const i8> = tmux_attach_args
        .iter()
        .map(|arg| arg.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();

    unsafe {
        execvp(tmux_attach.as_ptr(), tmux_attach_args_ptrs.as_ptr());
    }
    Err(anyhow::anyhow!("Unable to attach to tmux session!"))
}

fn tmux_has_session(session_name: &str) -> Result<bool> {
    let output = Command::new("tmux")
        .args(["has-session", "-t", session_name])
        .output()
        .with_context(|| "Failed to run tmux command.")?;
    Ok(output.status.success())
}

fn send_commands_to_session(session_name: &str, commands: &Vec<String>) -> Result<()> {
    for command in commands {
        run_tmux_command(&["send-keys", "-t", session_name, command, "C-m"])?;
    }
    Ok(())
}

fn get_layout_selection(twm_config: &TwmGlobal) -> Result<String> {
    let layouts_list: Vec<&str> = twm_config
        .layouts
        .keys()
        .map(std::convert::AsRef::as_ref)
        .collect();
    get_skim_selection_from_slice(&layouts_list, "Select a layout: ")
}

fn get_layout_to_use<'a>(
    workspace_type: Option<&str>,
    twm_config: &'a TwmGlobal,
    cli_config: &Arguments,
    local_config: &'a Option<TwmLocal>,
) -> Result<Option<&'a LayoutDefinition>> {
    // if user wants to choose a layout do this first
    if cli_config.layout {
        let layout_name = get_layout_selection(twm_config)?;
        return Ok(twm_config.layouts.get(&layout_name));
    }

    // next check if a local layout exists
    if let Some(local_layout) = local_config {
        return Ok(Some(&local_layout.layout));
    }

    match workspace_type {
        Some(t) => {
            if let Some(layout) = &twm_config
                .workspace_definitions
                .get(t)
                .expect("Workspace type not found!")
                .default_layout
            {
                Ok(twm_config.layouts.get(layout))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

pub fn open_workspace(
    workspace_path: &SafePath,
    workspace_type: Option<&str>,
    config: &TwmGlobal,
    args: &Arguments,
) -> Result<()> {
    let tmux_name = SessionName::from(workspace_path.path.as_str());
    if !tmux_has_session(&tmux_name.name)? {
        create_tmux_session(&tmux_name, workspace_path.path.as_str())?;
        let local_config = TwmLocal::load(Path::new(workspace_path.path.as_str()))?;
        let layout = get_layout_to_use(workspace_type, config, args, &local_config)?;
        if let Some(layout) = layout {
            send_commands_to_session(&tmux_name.name, &layout.commands)?;
        }
    }
    attach_to_tmux_session(&tmux_name.name)?;
    Ok(())
}
