use std::io;

use anyhow::{Context, Result};

pub fn get_skim_selection_from_slice(items: &[&str], prompt: &str) -> Result<String> {
    let opts = skim::prelude::SkimOptionsBuilder::default()
        .prompt(Some(prompt))
        .color(Some("blue"))
        .tiebreak(Some("score".to_string()))
        .tiebreak(Some("length".to_string()))
        .build()?;
    let item_reader = skim::prelude::SkimItemReader::default();
    let items_skim = items.join("\n");
    let receiver = item_reader.of_bufread(io::Cursor::new(items_skim));
    let result =
        skim::Skim::run_with(&opts, Some(receiver)).with_context(|| "Failed to run skim picker")?;
    if result.is_abort {
        anyhow::bail!("Skim finder aborted");
    }
    match result.selected_items.first() {
        Some(item) => Ok(item.output().to_string()),
        None => anyhow::bail!("No item selected from skim finder!"),
    }
}
