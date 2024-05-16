use std::collections::HashMap;
use std::path::Path;

use crate::picker::get_skim_selection_from_slice;
use anyhow::Result;

use crate::config::TwmGlobal;
use crate::matches::{find_workspaces_in_dir, SafePath};
use crate::tmux::{get_tmux_sessions, open_workspace, open_workspace_in_group};
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

    if args.group {
        let existing_sessions = get_tmux_sessions()?;
        let group_session_name = get_skim_selection_from_slice(
            &existing_sessions
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>(), // TODO: not ideal...
            "Select a session to group with: ",
        )?;
        open_workspace_in_group(&group_session_name, &args)?;
        Ok(())
    } else {
        let config = TwmGlobal::load()?;
        let mut matched_workspaces = HashMap::<SafePath, &str>::new();
        // handle a path directly being passed in first
        let workspace_path = if let Some(path) = args.path.clone() {
            let path_full = std::fs::canonicalize(path)?;
            match SafePath::try_from(path_full.as_path()) {
                Ok(p) => p,
                Err(_) => anyhow::bail!("Path is not valid UTF-8"),
            }
        } else {
            for dir in &config.search_paths {
                match SafePath::try_from(Path::new(dir)) {
                    Ok(_) => find_workspaces_in_dir(dir.as_str(), &config, &mut matched_workspaces),
                    Err(_) => {
                        anyhow::bail!("Path is not valid UTF-8: {}", dir);
                    }
                }
            }
            let workspace_name = get_skim_selection_from_slice(
                &matched_workspaces
                    .keys()
                    .map(|k| k.path.as_str())
                    .collect::<Vec<&str>>(),
                "Select a workspace: ",
            )?;
            SafePath::try_from(Path::new(workspace_name.as_str()))?
        };

        let workspace_type = matched_workspaces.get(&workspace_path).copied();
        open_workspace(&workspace_path, workspace_type, &config, &args)?;

        Ok(())
    }
}
