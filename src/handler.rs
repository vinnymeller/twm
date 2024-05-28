use std::path::Path;

use anyhow::Result;

use crate::{
    cli::Arguments,
    config::TwmGlobal,
    matches::find_workspaces_in_dir,
    picker::get_skim_selection_from_slice,
    tmux::{attach_to_tmux_session, get_tmux_sessions, open_workspace, open_workspace_in_group},
    workspace::get_workspace_type_for_path,
};

pub fn handle_existing_session_selection() -> Result<()> {
    let existing_sessions = get_tmux_sessions()?;
    let session_name =
        get_skim_selection_from_slice(&existing_sessions, "Select a session to attach to: ")?;
    attach_to_tmux_session(&session_name)?;
    Ok(())
}

pub fn handle_group_session_selection(args: &Arguments) -> Result<()> {
    let existing_sessions = get_tmux_sessions()?;
    let group_session_name =
        get_skim_selection_from_slice(&existing_sessions, "Select a session to group with: ")?;
    open_workspace_in_group(&group_session_name, args)?;
    Ok(())
}

pub fn handle_workspace_selection(args: &Arguments) -> Result<()> {
    let config = TwmGlobal::load()?;
    let mut matched_workspace_paths = Vec::<String>::new();
    // handle a path directly being passed in first
    let workspace_path = if let Some(path) = &args.path {
        let path_full = std::fs::canonicalize(path)?;
        match path_full.to_str() {
            Some(p) => p.to_owned(),
            None => anyhow::bail!("Path is not valid UTF-8"),
        }
    } else {
        for dir in &config.search_paths {
            find_workspaces_in_dir(dir.as_str(), &config, &mut matched_workspace_paths)
        }
        get_skim_selection_from_slice(&matched_workspace_paths, "Select a workspace: ")?
    };

    let workspace_type =
        get_workspace_type_for_path(Path::new(&workspace_path), &config.workspace_definitions);
    open_workspace(&workspace_path, workspace_type, &config, args)?;

    Ok(())
}
