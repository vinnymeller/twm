use std::collections::HashMap;
use std::path::Path;

use crate::picker::get_skim_selection_from_slice;
use anyhow::Result;

use crate::config::TwmGlobal;
use crate::matches::{find_workspaces_in_dir, SafePath};
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
