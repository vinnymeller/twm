use crate::config::TwmGlobal;
use std::collections::HashMap;
use std::path::PathBuf;

use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct WorkspaceMatch {
    pub path: PathBuf,
    pub path_lossy: String,
    pub workspace_definition_name: Option<String>,
}

pub fn find_workspaces_in_dir(
    dir: &str,
    config: &TwmGlobal,
    workspaces: &mut HashMap<String, WorkspaceMatch>,
) {
    let is_excluded = |entry: &DirEntry| -> bool {
        config.exclude_path_components.iter().any(|excl| {
            entry
                .path()
                .to_str()
                .expect("This shouldn't ever happen, I already checked!")
                .contains(excl)
        })
    };

    let walker = WalkDir::new(dir)
        .max_depth(config.max_search_depth)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir() && !is_excluded(e) && e.path().to_str().is_some())
        .filter_map(std::result::Result::ok);

    for entry in walker {
        let path = entry.path();
        let path_lossy = path.to_string_lossy().to_string();

        for workspace_definition in &config.workspace_definitions {
            for file_name in &workspace_definition.has_any_file {
                if path.join(file_name).exists() {
                    workspaces.insert(
                        path_lossy.clone(),
                        WorkspaceMatch {
                            path: path.to_path_buf(),
                            path_lossy: path_lossy.clone(),
                            workspace_definition_name: Some(workspace_definition.name.clone()),
                        },
                    );
                    break;
                }
            }
            if workspaces.contains_key(&path_lossy) {
                break;
            }
        }
    }
}
