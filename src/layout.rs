use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayoutDefinition {
    pub name: String,
    pub inherits: Option<Vec<String>>,
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
