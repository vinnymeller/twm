use crate::{
    handler::{
        handle_existing_session_selection, handle_group_session_selection,
        handle_make_default_config, handle_make_default_layout_config,
        handle_print_bash_completions, handle_print_config_schema, handle_print_fish_completions,
        handle_print_layout_config_schema, handle_print_man, handle_print_zsh_completions,
        handle_workspace_selection,
    },
    ui::Tui,
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
    /// Prompt user to select an existing tmux session to attach to.
    ///
    /// This shouldn't be used with other options.
    pub existing: bool,

    #[clap(short, long)]
    /// Prompt user to start a new session in the same group as an existing session.
    ///
    /// Setting this option will cause `-l/--layout` and `-p/--path` to be ignored.
    pub group: bool,

    #[clap(short, long)]
    #[clap(short = 'G')]
    /// Group with an existing session by name.
    pub group_with: Option<String>,

    #[clap(short, long)]
    /// Don't attach to the workspace session after opening it.
    pub dont_attach: bool,

    #[clap(short, long)]
    /// Prompt user to select a globally-defined layout to open the workspace with.
    ///
    /// Using this option will override any other layout definitions that would otherwise automatically be used when opening the workspace.
    pub layout: bool,

    #[clap(short, long)]
    /// Open the given path as a workspace.
    ///
    /// Using this option does not require that the path be a valid workspace according to your configuration.
    pub path: Option<String>,

    #[clap(short, long)]
    /// Force the workspace to be opened with the given name.
    ///
    /// When setting this option, you should be aware that twm will not "see" this session when performing other automatic actions.
    /// For example, if you have a workspace at ~/foobar and run `twm -n jimbob -p ~/foobar`, and then run `twm` and select `~/foobar` from the picker, a new session `foobar` will be created. If you then run `twm -g` and select `foobar`, `foobar-1` will be created in the `foobar` group.
    pub name: Option<String>,

    #[clap(short, long)]
    #[clap(short = 'N')]
    /// Print the name of the workspace generated for the given path to stdout.
    ///
    /// This can be used with other options.
    pub print_workspace_name: bool,

    #[clap(short, long)]
    /// Override any layouts and open the workspace with the given command instead.
    pub command: Option<String>,

    #[clap(long)]
    /// Make default configuration file.
    ///
    /// By default will attempt to write a default configuration file and configuration schema in `$XDG_CONFIG_HOME/twm/`
    /// Using `-p/--path` with this flag will attempt to write the files to the folder specified.
    /// twm will not overwrite existing files. You will be prompted to rename/move the existing files before retrying.
    pub make_default_config: bool,

    #[clap(long)]
    /// Make default local layout configuration file.
    ///
    /// Will attempt to create `.twm.yaml` in the current directory. Will not overwrite existing files.
    /// You can use `-p/--path <PATH>` to specify a different directory to write the file to.
    pub make_default_layout_config: bool,

    #[clap(long)]
    /// Print the configuration file (twm.yaml) schema.
    ///
    /// This can be used with tools (e.g. language servers) to provide autocompletion and validation when editing your configuration.
    pub print_config_schema: bool,

    #[clap(long)]
    /// Print the local layout configuration file (.twm.yaml) schema.
    ///
    /// This can be used with tools (e.g. language servers) to provide autocompletion and validation when editing your configuration.
    pub print_layout_config_schema: bool,

    #[clap(long)]
    /// Print bash completions to stdout
    pub print_bash_completion: bool,

    #[clap(long)]
    /// Print zsh completions to stdout
    pub print_zsh_completion: bool,

    #[clap(long)]
    /// Print fish completions to stdout
    pub print_fish_completion: bool,

    #[clap(long)]
    /// Print man(1) page to stdout
    pub print_man: bool,
}

/// Parses the command line arguments and runs the program. Called from `main.rs`.
/// Since not every command needs a TUI, we start one up as necessary in each handler that needs one.
pub fn parse() -> Result<()> {
    let args = Arguments::parse();

    // This kind of matching couuld be avoided by using subcommands but I just generally like flags better.
    // Who's going to try running `twm --group --print-man --print-config-schema` anyways? grow up
    match args {
        Arguments {
            make_default_config: true,
            ..
        } => handle_make_default_config(&args),
        Arguments {
            make_default_layout_config: true,
            ..
        } => handle_make_default_layout_config(&args),
        Arguments {
            print_config_schema: true,
            ..
        } => handle_print_config_schema(),
        Arguments {
            print_layout_config_schema: true,
            ..
        } => handle_print_layout_config_schema(),
        Arguments {
            print_bash_completion: true,
            ..
        } => handle_print_bash_completions(),
        Arguments {
            print_zsh_completion: true,
            ..
        } => handle_print_zsh_completions(),
        Arguments {
            print_fish_completion: true,
            ..
        } => handle_print_fish_completions(),
        Arguments {
            print_man: true, ..
        } => handle_print_man(),
        _ => {
            let mut tui = Tui::start()?;
            let res = if args.existing {
                handle_existing_session_selection(&args, &mut tui)
            } else if args.group {
                handle_group_session_selection(&args, &mut tui)
            } else {
                handle_workspace_selection(&args, &mut tui)
            };
            tui.exit()?;
            res
        }
    }
}
