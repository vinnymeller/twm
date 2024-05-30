use crate::config::TwmGlobal;
use crate::workspace::path_meets_workspace_conditions;

use nucleo::Injector;
use walkdir::{DirEntry, WalkDir};

pub fn find_workspaces_in_dir(dir: &str, config: &TwmGlobal, injector: Injector<String>) {
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
                // skip here instead of checking earlier because i don't expect people having a bunch of non-utf8 paths to be common, so defer the check only if we have a match in the first place
                if let Some(utf8_path) = entry.path().to_str() {
                    // previously we also stored which workspace type we matched on, but i decided to change it because we only ever need to know the workspace type for the workspace we're opening anyways
                    // having to re-lookup the workspace type on user selection is surely better than the hashmap we were using before, but better would probably be to just keep track of which WorkspaceDefinition matched here
                    // main reason I haven't yet is because I'm not entirely sure how to make that work nicely with the fuzzy finders
                    injector.push(utf8_path.to_string(), |_, dst| {
                        dst[0] = utf8_path.to_string().into()
                    });
                }
                break;
            }
        }
    }
}
