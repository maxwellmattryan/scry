use std::fs;
use std::io::Write;

use crate::deck::{Deck, ManaBase};

pub struct JsonExporter;

impl JsonExporter {
    pub fn export(
        deck: &Deck,
        mana_base: &ManaBase,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = serde_json::json!({
            "deck": deck,
            "mana_base": mana_base,
            "generated_at": chrono::Local::now().to_rfc3339(),
        });

        let json = serde_json::to_string_pretty(&output)?;
        let mut file = fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
