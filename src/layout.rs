use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayoutDefinition {
    /// Name of the layout.
    ///
    /// This is the name that should be referenced in workspace definitions' `default_layout` field.
    pub name: String,

    /// List of layout names that this layout should inherit commands from.
    ///
    /// If unset, commands are not inherited from any other layouts.
    ///
    /// Commands are inherited in the order they are listed.
    ///
    /// Only layouts in the main `twm.yaml` configuration file can be used in this list. There is no way for twm
    /// to find all of the local layouts that might exist in other workspaces.
    ///
    /// This is useful when you want to share complex base layouts that might slightly differ between different types of
    /// workspaces. For example, you might define a complicated layout with 5 windows and 20 panes, but want to run
    /// different commands in some panes Python workspaces than in Rust workspaces. You could define the window & pane
    /// layout in a base layout and inherit from it in your Python and Rust layouts, simply using the `commands` field
    /// to run the workspace-specific commands for each respective workspace type.
    pub inherits: Option<Vec<String>>,

    /// List of commands to run when a session using this layout is initialized.
    ///
    /// If unset, no commands are run when the session is initialized.
    ///
    /// Commands defined here are run after commands from inherited layouts.
    ///
    /// These commands are passed to the  shell as-is via tmux's `send-keys` command.
    pub commands: Option<Vec<String>>,
}

pub fn get_layout_by_name<'a>(
    name: &str,
    layouts: &'a [LayoutDefinition],
) -> Option<&'a LayoutDefinition> {
    layouts.iter().find(|l| l.name == name)
}

pub fn get_commands_from_layout<'a: 'c, 'b: 'c, 'c>(
    layout: &'a LayoutDefinition,
    layouts: &'b [LayoutDefinition],
) -> Vec<&'c str> {
    let mut commands = Vec::<&str>::new();
    if let Some(inherits_list) = &layout.inherits {
        for inherits_from_name in inherits_list {
            commands.extend(get_commands_from_layout_name(inherits_from_name, layouts));
        }
    }
    if let Some(layout_commands) = &layout.commands {
        commands.extend(layout_commands.iter().map(String::as_str));
    }
    commands
}

pub fn get_commands_from_layout_name<'a: 'c, 'b: 'c, 'c>(
    layout_name: &'a str,
    layouts: &'b [LayoutDefinition],
) -> Vec<&'c str> {
    match get_layout_by_name(layout_name, layouts) {
        Some(layout) => get_commands_from_layout(layout, layouts),
        None => Vec::new(),
    }
}

pub fn get_layout_names(layouts: &[LayoutDefinition]) -> Vec<String> {
    layouts.iter().map(|l| l.name.clone()).collect()
}
