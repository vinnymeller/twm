use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Deserialize, Debug)]
pub struct WorkspaceDefinition {
    pub name: String,
    pub has_any_file: Vec<String>,
    pub default_layout: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LayoutDefinition {
    pub name: String,
    pub commands: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct RawTwmGlobal {
    search_paths: Option<Vec<String>>,
    workspace_definitions: Option<Vec<WorkspaceDefinition>>,
    max_search_depth: Option<usize>,
    exclude_path_components: Option<Vec<String>>,
    layouts: Option<Vec<LayoutDefinition>>,
}

#[derive(Debug)]
pub struct TwmGlobal {
    pub search_paths: Vec<String>,
    pub exclude_path_components: Vec<String>,
    pub workspace_definitions: Vec<WorkspaceDefinition>, // preserve order of insertion since order is implicitly the priority
    pub layouts: Vec<LayoutDefinition>,
    pub max_search_depth: usize,
}

#[derive(Debug, Deserialize)]
pub struct TwmLocal {
    pub layout: LayoutDefinition,
}

impl TryFrom<RawTwmGlobal> for TwmGlobal {
    type Error = anyhow::Error;

    fn try_from(raw_config: RawTwmGlobal) -> std::result::Result<Self, Self::Error> {
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
            Some(workspace_definitions) => workspace_definitions,
            None => vec![WorkspaceDefinition {
                name: String::from("default"),
                has_any_file: vec![".git".to_string()],
                default_layout: None,
            }],
        };

        let layouts = raw_config.layouts.unwrap_or_default();

        let max_search_depth = raw_config.max_search_depth.unwrap_or(3);

        let config = TwmGlobal {
            search_paths,
            exclude_path_components,
            workspace_definitions,
            layouts,
            max_search_depth,
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

impl FromStr for TwmLocal {
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

impl TwmLocal {
    pub fn load(path: &Path) -> Result<Option<Self>> {
        const CONFIG_FILE_NAME: &str = ".twm.yaml";
        let config_path = path.join(CONFIG_FILE_NAME);
        if config_path.exists() {
            let config = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from path: {config_path:#?}"))?;
            Ok(Some(TwmLocal::from_str(&config)?))
        } else {
            Ok(None)
        }
    }
}

pub fn load() -> Result<TwmGlobal> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix(clap::crate_name!())
        .with_context(|| "Failed to load XDG dirs.")?;
    let config_file_name = format!("{}.yaml", clap::crate_name!());
    let config_path = xdg_dirs.find_config_file(&config_file_name);
    let raw_config = match config_path {
        Some(path) => RawTwmGlobal::try_from(&path),
        None => RawTwmGlobal::from_str(""),
    }?;
    let config = TwmGlobal::try_from(raw_config)
        .with_context(|| "Failed to validate configuration settings.")?;
    Ok(config)
}
