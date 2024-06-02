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
    pub name: String,
    pub has_any_file: Option<Vec<String>>,
    pub has_all_files: Option<Vec<String>>,
    pub missing_any_file: Option<Vec<String>>,
    pub missing_all_files: Option<Vec<String>>,
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
    search_paths: Option<Vec<String>>,
    workspace_definitions: Option<Vec<WorkspaceDefinitionConfig>>,
    max_search_depth: Option<usize>,
    session_name_path_components: Option<usize>,
    exclude_path_components: Option<Vec<String>>,
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
