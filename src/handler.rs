use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{crate_name, CommandFactory};
use clap_complete::{generate, Shell};

use crate::{
    cli::Arguments,
    config::{RawTwmGlobal, TwmGlobal, TwmLayout},
    matches::find_workspaces_in_dir,
    tmux::{
        attach_to_tmux_session, get_tmux_sessions, open_workspace, open_workspace_in_group,
        session_name_for_path_recursive,
    },
    workspace::get_workspace_type_for_path,
};

use crate::ui::picker::{Picker, PickerSelection};

fn print_completion(shell: Shell) -> Result<()> {
    let mut cmd = Arguments::command();
    generate(shell, &mut cmd, crate_name!(), &mut std::io::stdout());
    Ok(())
}

pub fn handle_print_bash_completions() -> Result<()> {
    print_completion(Shell::Bash)
}

pub fn handle_print_zsh_completions() -> Result<()> {
    print_completion(Shell::Zsh)
}

pub fn handle_print_fish_completions() -> Result<()> {
    print_completion(Shell::Fish)
}

pub fn handle_print_schema() -> Result<()> {
    println!("{}", RawTwmGlobal::schema()?);
    Ok(())
}

pub fn handle_print_layout_schema() -> Result<()> {
    println!("{}", TwmLayout::schema()?);
    Ok(())
}

pub fn handle_make_default_config(args: &Arguments) -> Result<()> {
    let config_filename = format!("{}.yaml", crate_name!());
    let schema_filename = format!("{}.schema.json", crate_name!());
    let (config_path, schema_path) = if args.path.is_some() {
        let mut path = PathBuf::from(args.path.as_ref().expect("Path was just checked?"));
        if path.is_file() {
            path.pop();
        }
        (path.join(&config_filename), path.join(&schema_filename))
    } else {
        let base_dirs = xdg::BaseDirectories::with_prefix(crate_name!())?;
        (
            base_dirs.get_config_file(&config_filename),
            base_dirs.get_config_file(&schema_filename),
        )
    };

    if config_path.exists() || schema_path.exists() {
        anyhow::bail!(format!(
            "Configuration files already exist. Please move or rename any existing files:
- {}
- {}
before running this command again.",
            config_path.display(),
            schema_path.display()
        ));
    }

    // make sure parent directories exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // write the schema to the schema path
    std::fs::write(schema_path, RawTwmGlobal::schema()?)?;
    std::fs::write(
        config_path,
        format!(
            r#"# yaml-language-server: $schema=./{}
search_paths:
  - "~"

exclude_path_components:
  - .git
  - .direnv
  - .cargo
  - node_modules
  - venv
  - target
  - __pycache__

max_search_depth: 5
session_name_path_components: 2

workspace_definitions:
  - name: default
    has_any_file:
      - .git
      - .twm.yaml
    default_layout: default

layouts:
  - name: default
    commands:
      - echo "twm session created in $TWM_ROOT"

"#,
            schema_filename
        ),
    )?;
    Ok(())
}

pub fn handle_existing_session_selection() -> Result<()> {
    let existing_sessions = get_tmux_sessions()?;
    let session_name = match Picker::new(
        &existing_sessions,
        "Select an existing session to attach to: ".into(),
    )
    .get_selection()?
    {
        PickerSelection::None => anyhow::bail!("No session selected"),
        PickerSelection::Selection(s) => s,
        PickerSelection::ModifiedSelection(s) => s,
    };
    attach_to_tmux_session(&session_name)?;
    Ok(())
}

pub fn handle_group_session_selection(args: &Arguments) -> Result<()> {
    let existing_sessions = get_tmux_sessions()?;
    let group_session_name = match Picker::new(
        &existing_sessions,
        "Select a session to group with: ".into(),
    )
    .get_selection()?
    {
        PickerSelection::None => anyhow::bail!("No session selected"),
        PickerSelection::Selection(s) => s,
        PickerSelection::ModifiedSelection(s) => s,
    };
    open_workspace_in_group(&group_session_name, args)?;
    Ok(())
}

pub fn handle_workspace_selection(args: &Arguments) -> Result<()> {
    let config = TwmGlobal::load()?;
    let (workspace_path, try_grouping) = if let Some(path) = &args.path {
        let path_full = std::fs::canonicalize(path)?;
        match path_full.to_str() {
            Some(p) => (p.to_owned(), false),
            None => anyhow::bail!("Path is not valid UTF-8"),
        }
    } else {
        let mut picker = Picker::new(&[], "Select a workspace: ".into());
        let injector = picker.injector.clone();
        let config = config.clone();
        std::thread::spawn(move || {
            for dir in &config.search_paths {
                find_workspaces_in_dir(dir, &config, injector.clone())
            }
        });
        match picker.get_selection()? {
            PickerSelection::None => anyhow::bail!("No workspace selected"),
            PickerSelection::Selection(s) => (s, false),
            PickerSelection::ModifiedSelection(s) => (s, true),
        }
    };

    if try_grouping {
        // see if we already have a twm-generated session for the workspace path we're trying to open
        if let Ok(Some(group_session_name)) =
            session_name_for_path_recursive(&workspace_path, config.session_name_path_components)
        {
            open_workspace_in_group(group_session_name.as_str(), args)?;
            return Ok(());
        }
    }

    // if we couldn't find a correct session to group with, open the workspace normally

    let workspace_type =
        get_workspace_type_for_path(Path::new(&workspace_path), &config.workspace_definitions);
    open_workspace(&workspace_path, workspace_type, &config, args)?;

    Ok(())
}
