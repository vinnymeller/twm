use crate::config::TwmGlobal;
use crate::workspace_conditions::path_meets_workspace_conditions;
use anyhow::Result;
use std::path::Path;

use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SafePath {
    pub path: String,
}

impl TryFrom<&Path> for SafePath {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self> {
        match path.to_str() {
            Some(s) => Ok(Self {
                path: s.to_string(),
            }),
            None => anyhow::bail!("Path is not valid UTF-8"),
        }
    }
}

impl SafePath {
    #[must_use]
    pub fn new(str: &str) -> Self {
        Self {
            path: str.to_string(),
        }
    }
}

pub fn find_workspaces_in_dir(dir: &str, config: &TwmGlobal, workspaces: &mut Vec<String>) {
    let is_excluded = |entry: &DirEntry| -> bool {
        match entry
            .path()
            .components()
            .last()
            .expect("Surely there is always a last component?")
            .as_os_str()
            .to_str()
        {
            Some(s) => config.exclude_path_components.iter().any(|e| s == e),
            None => true,
        }
    };

    let walker = WalkDir::new(dir)
        .max_depth(config.max_search_depth)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir() && !is_excluded(e))
        .filter_map(std::result::Result::ok);

    for entry in walker {
        let path = match SafePath::try_from(entry.path()) {
            Ok(p) => p,
            Err(_) => continue,
        };

        for workspace_definition in &config.workspace_definitions {
            if path_meets_workspace_conditions(entry.path(), &workspace_definition.conditions) {
                workspaces.push(path.path);
                break;
            }
        }
    }
}
