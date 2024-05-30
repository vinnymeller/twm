use crate::handler::{
    handle_existing_session_selection, handle_group_session_selection, handle_make_default_config,
    handle_print_layout_schema, handle_print_schema, handle_workspace_selection,
};
use anyhow::Result;

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
    /// Prompt user to select an existing tmux session to attach to.
    ///
    /// This nullifies all other options.
    pub existing: bool,

    #[clap(short, long)]
    /// Prompt user to start a new session in the same group as an existing session.
    ///
    /// Setting this option nullifies the layout and path options.
    pub group: bool,

    #[clap(short, long)]
    /// Open the given path as a workspace.
    ///
    /// Using this option does not require that the path be a valid workspace according to your configuration.
    pub path: Option<String>,

    #[clap(short, long)]
    /// Force the workspace to be opened with the given name.
    ///
    /// twm will not store any knowledge of the fact that you manually named the workspace. I.e. if you open the workspace at path `/home/user/dev/api` and name it `jimbob`, and then open the same workspace again manually, you will have two instances of the workspace open with different names.
    pub name: Option<String>,

    #[clap(short, long)]
    /// Don't attach to the workspace session after opening it.
    pub dont_attach: bool,

    #[clap(long)]
    /// Print the configuration file schema.
    ///
    /// This can be used with tools (e.g. language servers) to provide autocompletion and validation when editing your configuration.
    pub print_schema: bool,

    #[clap(long)]
    /// Print the local layout configuration file schema.
    ///
    /// This can be used with tools (e.g. language servers) to provide autocompletion and validation when editing your configuration.
    pub print_layout_schema: bool,

    #[clap(long)]
    /// Make default configuration file.
    ///
    /// By default will attempt to write a default configuration file and configuration schema in `$XDG_CONFIG_HOME/twm/`
    /// Using `-p/--path` with this flag will attempt to write the files to the folder specified.
    pub make_default_config: bool,
}

/// Parses the command line arguments and runs the program. Called from `main.rs`.
pub fn parse() -> Result<()> {
    let args = Arguments::parse();

    match args {
        Arguments {
            make_default_config: true,
            ..
        } => handle_make_default_config(&args),
        Arguments {
            print_schema: true, ..
        } => handle_print_schema(),
        Arguments {
            print_layout_schema: true,
            ..
        } => handle_print_layout_schema(),
        Arguments { existing: true, .. } => handle_existing_session_selection(),
        Arguments { group: true, .. } => handle_group_session_selection(&args),
        _ => handle_workspace_selection(&args),
    }
}
