use std::path::Path;

use crate::picker::get_skim_selection_from_slice;
use crate::workspace::get_workspace_type_for_path;
use anyhow::Result;

use crate::config::TwmGlobal;
use crate::matches::find_workspaces_in_dir;
use crate::tmux::{
    attach_to_tmux_session, get_tmux_sessions, open_workspace, open_workspace_in_group,
};
use clap::Parser;

#[derive(Parser, Default, Debug)]
#[clap(author = "Vinny Meller", version)]
/// twm (tmux workspace manager) is a customizable tool for managing workspaces in tmux sessions.
///
/// Workspaces are defined as a directory matching any workspace pattern from your configuration. If no configuration is set, any directory containing a `.git` file/folder or a `.twm.yaml` file is considered a workspace.
pub struct Arguments {
    #[clap(short, long)]
    /// Prompt user to select a globally-defined layout to open the workspace with.
    ///
    /// Using this option will override any other layout definitions.
    pub layout: bool,

    #[clap(short, long)]
    /// Prompt user to select an existing tmux session to attach to.
    ///
    /// This nullifies all other options.
    pub existing: bool,

    #[clap(short, long)]
    /// Prompt user to start a new session in the same group as an existing session.
    ///
    /// Setting this option nullifies the layout and path options.
    pub group: bool,

    #[clap(short, long)]
    /// Open the given path as a workspace.
    ///
    /// Using this option does not require that the path be a valid workspace according to your configuration.
    path: Option<String>,

    #[clap(short, long)]
    /// Force the workspace to be opened with the given name.
    ///
    /// twm will not store any knowledge of the fact that you manually named the workspace. I.e. if you open the workspace at path `/home/user/dev/api` and name it `jimbob`, and then open the same workspace again manually, you will have two instances of the workspace open with different names.
    pub name: Option<String>,

    #[clap(short, long)]
    /// Don't attach to the workspace session after opening it.
    pub dont_attach: bool,
}

/// Parses the command line arguments and runs the program. Called from `main.rs`.
pub fn parse() -> Result<()> {
    let args = Arguments::parse();

    if args.existing {
        let existing_sessions = get_tmux_sessions()?;
        let session_name =
            get_skim_selection_from_slice(&existing_sessions, "Select a session to attach to: ")?;
        attach_to_tmux_session(&session_name)?;
        Ok(())
    } else if args.group {
        let existing_sessions = get_tmux_sessions()?;
        let group_session_name =
            get_skim_selection_from_slice(&existing_sessions, "Select a session to group with: ")?;
        open_workspace_in_group(&group_session_name, &args)?;
        Ok(())
    } else {
        let config = TwmGlobal::load()?;
        let mut matched_workspace_paths = Vec::<String>::new();
        // handle a path directly being passed in first
        let workspace_path = if let Some(path) = args.path.clone() {
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
        open_workspace(&workspace_path, workspace_type, &config, &args)?;

        Ok(())
    }
}
