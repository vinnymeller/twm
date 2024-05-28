use crate::config::TwmGlobal;
use crate::workspace_conditions::path_meets_workspace_conditions;

use walkdir::{DirEntry, WalkDir};

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
        for workspace_definition in &config.workspace_definitions {
            if path_meets_workspace_conditions(entry.path(), &workspace_definition.conditions) {
                // just skip the path if it's not valid utf-8 since we can't use it
                if let Some(utf8_path) = entry.path().to_str() {
                    workspaces.push(utf8_path.to_string());
                }
                break;
            }
        }
    }
}
