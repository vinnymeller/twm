use std::collections::HashMap;

use crate::picker::get_skim_selection_from_slice;
use anyhow::Result;

use crate::config;
use crate::matches::{find_workspaces_in_dir, WorkspaceMatch};
use crate::tmux::open_workspace;
use clap::Parser;

#[derive(Parser, Default, Debug)]
#[clap(author = "Vinny Meller", version, about)]
/// A utility for managing workspaces in tmux.
pub struct Arguments {
    #[clap(short, long)]
    /// Prompt user to select a layout to open the workspace with.
    pub layout: bool,

    #[clap(short, long)]
    /// Open the given path as a workspace.
    path: Option<String>,
}

pub fn parse() -> Result<()> {
    let args = Arguments::parse();

    let mut config = config::load()?;

    // handle a path directly being passed in first
    let workspace_match = if let Some(path) = args.path.clone() {
        let path_full = std::fs::canonicalize(path)?;
        let path_match = WorkspaceMatch {
            path: path_full.clone(),
            path_lossy: path_full.to_string_lossy().to_string(),
            workspace_definition_name: None,
        };
        path_match
    } else {
        let mut matched_workspaces = HashMap::<String, WorkspaceMatch>::new();
        for dir in &config.search_paths {
            find_workspaces_in_dir(dir.as_str(), &config, &mut matched_workspaces);
        }
        let workspace_name = get_skim_selection_from_slice(
            &matched_workspaces
                .keys()
                .map(std::convert::AsRef::as_ref)
                .collect::<Vec<&str>>(),
            "Select a workspace: ",
        )?;
        match matched_workspaces.remove(&workspace_name) {
            Some(workspace_match) => workspace_match,
            None => anyhow::bail!("Failed to find workspace match for {}", workspace_name),
        }
    };
    open_workspace(&workspace_match, &mut config, &args)?;

    Ok(())
}
