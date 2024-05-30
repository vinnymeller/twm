use std::path::Path;

use anyhow::Result;

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

pub fn handle_print_schema() -> Result<()> {
    RawTwmGlobal::print_schema()
}

pub fn handle_print_layout_schema() -> Result<()> {
    TwmLayout::print_schema()
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
