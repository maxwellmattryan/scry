#![allow(dead_code)]

use serde::Serialize;
use std::fs;

/// Generic JSON exporter for any serializable type
pub struct JsonExporter;

impl JsonExporter {
    /// Export data to a JSON file
    pub fn export<T: Serialize>(data: &T, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(data)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Generate JSON string without writing to file
    pub fn generate<T: Serialize>(data: &T) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}
