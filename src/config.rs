// TODO: figure out how to handle turning the config file into the final structs used
// throughout the program. this shit is a mess!!

use crate::layout::LayoutDefinition;
use crate::workspace::{
    HasAnyFileCondition, MissingAllFilesCondition, MissingAnyFileCondition, NullCondition,
    WorkspaceConditionEnum, WorkspaceDefinition,
};
use anyhow::{Context, Result};
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
struct WorkspaceDefinitionConfig {
    /// Name for the workspace type defined by the list item.
    ///
    /// This name corresponds to the `TWM_TYPE` environment variable that will be set inside a session.
    pub name: String,

    /// List of files for which at least one must be present in a directory to be considered a workspace of this type.
    ///
    /// If unset, this constraint is simply ignored.
    ///
    /// For example if the list is `["requirements.txt", "Pipfile", "pyproject.toml", "poetry.lock", "setup.py"]`, a
    /// directory not containing *any* of those files cannot match this workspace definition.
    pub has_any_file: Option<Vec<String>>,

    /// List of files for which all must be present in a directory to be considered a workspace of this type.
    ///
    /// If unset, this constraint is simply ignored.
    ///
    /// For example, if the list is `["flake.nix", ".envrc"]`, only directories with *both* files present can match
    /// this workspace definition.
    pub has_all_files: Option<Vec<String>>,

    /// List of files for which at least one must be missing in a directory to be considered a workspace of this type.
    ///
    /// If unset, this constraint is simply ignored.
    ///
    /// For example, if the list is `["node_modules", "target"]`, directories containing *both* `node_modules` and `target`
    /// cannot match this workspace definition.
    pub missing_any_file: Option<Vec<String>>,

    /// List of files for which all must be missing in a directory to be considered a workspace of this type.
    ///
    /// If unset, this constraint is simply ignored.
    ///
    /// For example, if the list is `["node_modules", "target"]`, directories containing *either* `node_modules` or `target`
    /// cannot match this workspace definition.
    pub missing_all_files: Option<Vec<String>>,

    /// The name of the layout to apply to a session during initialization.
    ///
    /// If unset, no layout will be applied by default.
    ///
    /// This option can be overridden either by using the `-l/--layout` command line flag, which will prompt you to select
    /// a layout from the list of configured layouts, or by the presence of a `.twm.yaml` local layout configuration file
    /// in the workspace directory.
    pub default_layout: Option<String>,
}

impl From<WorkspaceDefinitionConfig> for WorkspaceDefinition {
    fn from(config: WorkspaceDefinitionConfig) -> Self {
        let mut conditions = Vec::<WorkspaceConditionEnum>::new();

        if let Some(has_any_file) = config.has_any_file {
            if !has_any_file.is_empty() {
                let condition = HasAnyFileCondition {
                    files: has_any_file,
                };
                conditions.push(condition.into());
            }
        }

        if let Some(has_all_files) = config.has_all_files {
            if !has_all_files.is_empty() {
                let condition = HasAnyFileCondition {
                    files: has_all_files,
                };
                conditions.push(condition.into());
            }
        }

        if let Some(missing_any_file) = config.missing_any_file {
            if !missing_any_file.is_empty() {
                let condition = MissingAnyFileCondition {
                    files: missing_any_file,
                };
                conditions.push(condition.into());
            }
        }

        if let Some(missing_all_files) = config.missing_all_files {
            if !missing_all_files.is_empty() {
                let condition = MissingAllFilesCondition {
                    files: missing_all_files,
                };
                conditions.push(condition.into());
            }
        }

        if conditions.is_empty() {
            let condition = NullCondition {};
            conditions.push(condition.into());
        }

        WorkspaceDefinition {
            name: config.name,
            conditions,
            default_layout: config.default_layout,
        }
    }
}

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawTwmGlobal {
    /// List of directories to have twm search for workspaces.
    ///
    /// If unset, defaults to `~` (shell expansion is supported).
    ///
    /// Be careful to not make your search paths overlap, e.g. if you include `~/projects` and `~/projects/foo/bar`
    /// with `max_search_depth: 3`, `~/projects/foo/bar` will be searched twice and results will be displayed twice
    /// in the picker. Generally it's easiest to just include the parent directory and increase `max_search_depth`
    /// if needed.
    search_paths: Option<Vec<String>>,

    /// List of configurations for workspaces.
    ///
    /// If unset, the default twm workspace definition is any directory containing a `.git` file/directory or a
    /// `.twm.yaml` layout file.
    ///
    /// When a directory is found that matches a workspace definition the first match, in order of appearance in
    /// this list, is the workspace "type" that will be for things like choosing which layout to apply to the session
    /// and in setting the `TWM_TYPE` environment variable
    workspace_definitions: Option<Vec<WorkspaceDefinitionConfig>>,

    /// Maximum depth to search for workspaces inside the `search_paths` directories.
    /// If unset, defaults to 3.
    max_search_depth: Option<usize>,

    /// Default number of components of the workspace directory to use for the created session name.
    /// If unset, defaults to 1.
    ///
    /// E.g. if you open a workspace at `/home/vinny/projects/foo/bar` and `session_name_path_components` is set to 1,
    /// The session name will be `bar`. If 2, `foo/bar`, etc.
    session_name_path_components: Option<usize>,

    /// List of path components which will *exclude* a directory from being considered a workspace.
    /// If unset, defaults to an empty list.
    ///
    /// A common use case would be to exclude things like `node_modules`, `target`, `__pycache__`, etc.
    exclude_path_components: Option<Vec<String>>,

    /// List of layout definitions made available when opening a workspace.
    /// If unset, defaults to an empty list.
    ///
    /// The layouts in this list can be used as the `default_layout` in a workspace definition and also
    /// will be available in the layout list when using `-l/--layout` command line flag.
    layouts: Option<Vec<LayoutDefinition>>,
}

impl RawTwmGlobal {
    pub fn schema() -> Result<String> {
        Ok(serde_json::to_string_pretty(&schema_for!(Self))?)
    }
}

#[derive(Debug, Clone)]
pub struct TwmGlobal {
    pub search_paths: Vec<String>,
    pub exclude_path_components: Vec<String>,
    pub workspace_definitions: Vec<WorkspaceDefinition>,
    pub session_name_path_components: usize,
    pub layouts: Vec<LayoutDefinition>,
    pub max_search_depth: usize,
}

#[derive(Debug, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TwmLayout {
    /// Layout definition to default to when opening the current workspace.
    /// This will override the `default_layout` in the matching workspace definition if present.
    pub layout: LayoutDefinition,
}

impl TwmLayout {
    pub fn schema() -> Result<String> {
        Ok(serde_json::to_string_pretty(&schema_for!(Self))?)
    }
}

impl TryFrom<RawTwmGlobal> for TwmGlobal {
    type Error = anyhow::Error;

    fn try_from(raw_config: RawTwmGlobal) -> Result<Self> {
        // search paths are the only place we need to worry about shell expansion
        let search_paths = match raw_config.search_paths {
            Some(paths) => paths,
            None => vec![String::from("~")],
        };

        let search_paths: Vec<String> = search_paths
            .iter()
            .map(|path| shellexpand::tilde(path).to_string())
            .collect();

        let exclude_path_components = raw_config.exclude_path_components.unwrap_or_default();

        let workspace_definitions = match raw_config.workspace_definitions {
            Some(workspace_definitions) => workspace_definitions
                .into_iter()
                .map(WorkspaceDefinition::from)
                .collect(),
            None => vec![WorkspaceDefinition {
                name: String::from("default"),
                conditions: vec![HasAnyFileCondition {
                    files: vec![".git".to_string()],
                }
                .into()],
                default_layout: None,
            }],
        };

        let layouts = raw_config.layouts.unwrap_or_default();

        let max_search_depth = raw_config.max_search_depth.unwrap_or(3);
        let session_name_path_components = raw_config.session_name_path_components.unwrap_or(1);

        let config = TwmGlobal {
            search_paths,
            exclude_path_components,
            workspace_definitions,
            layouts,
            max_search_depth,
            session_name_path_components,
        };

        Ok(config)
    }
}

impl TryFrom<&PathBuf> for RawTwmGlobal {
    type Error = anyhow::Error;

    fn try_from(path: &PathBuf) -> Result<Self> {
        let config = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from path: {path:#?}"))?;
        let raw_config =
            RawTwmGlobal::from_str(&config).with_context(|| "Failed to parse twm config file.")?;
        Ok(raw_config)
    }
}

impl FromStr for RawTwmGlobal {
    type Err = anyhow::Error;

    fn from_str(config: &str) -> Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::from_str(config, config::FileFormat::Yaml))
            .build()
            .with_context(|| "Failed build configuration. You should never see this. I think.")?;

        let raw_config = settings
            .try_deserialize()
            .with_context(|| "Failed to deserialize twm config.")?;
        Ok(raw_config)
    }
}

impl TwmGlobal {
    pub fn load() -> Result<Self> {
        let config_file_name = format!("{}.yaml", clap::crate_name!());
        let config_path = match std::env::var_os("TWM_CONFIG_FILE") {
            // if TWM_CONFIG_FILE is not set, search xdg dirs for config file as normal
            None => {
                let xdg_dirs = xdg::BaseDirectories::with_prefix(clap::crate_name!())
                    .with_context(|| "Failed to load XDG dirs.")?;
                xdg_dirs.find_config_file(config_file_name)
            }
            // if TWM_CONFIG_FILE is set, read from there no questions asked
            Some(config_file_path) => Some(PathBuf::from(config_file_path)),
        };
        let raw_config = match config_path {
            Some(path) => RawTwmGlobal::try_from(&path),
            None => RawTwmGlobal::from_str(""),
        }?;
        let config = TwmGlobal::try_from(raw_config)
            .with_context(|| "Failed to validate configuration settings.")?;
        Ok(config)
    }
}

impl FromStr for TwmLayout {
    type Err = anyhow::Error;

    fn from_str(config: &str) -> Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::from_str(config, config::FileFormat::Yaml))
            .build()
            .with_context(|| {
                "Failed to build configuration. You should never see this. I think."
            })?;

        let local_config = settings
            .try_deserialize()
            .with_context(|| "Failed to deserialize local twm config.")?;
        Ok(local_config)
    }
}

impl TwmLayout {
    /// Attemps to load a local config file from the given path.
    /// Will return Ok(None) if no config file is found.
    /// Errors if the config file is found but results in an error during parsing.
    pub fn load(path: &Path) -> Result<Option<Self>> {
        const CONFIG_FILE_NAME: &str = ".twm.yaml";
        let config_path = path.join(CONFIG_FILE_NAME);
        if config_path.exists() {
            let config = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from path: {config_path:#?}"))?;
            Ok(Some(TwmLayout::from_str(&config)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_empty_config_is_valid() {
        let raw_config = RawTwmGlobal::from_str("").unwrap();
        let _ = TwmGlobal::try_from(raw_config).unwrap();
    }
}
