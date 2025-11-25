use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single face of a double-faced or split card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardFace {
    pub name: String,
    pub mana_cost: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub colors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub mana_cost: Option<String>,
    pub cmc: f64,
    #[serde(default)]
    pub type_line: String,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub colors: Option<Vec<String>>,
    pub color_identity: Vec<String>,
    pub set: String,
    pub set_name: String,
    pub rarity: String,
    pub prices: Option<Prices>,
    pub legalities: HashMap<String, String>,
    pub image_uris: Option<ImageUris>,
    pub scryfall_uri: String,
    /// For double-faced cards, split cards, etc.
    pub card_faces: Option<Vec<CardFace>>,
    /// Layout type (normal, transform, modal_dfc, split, etc.)
    pub layout: Option<String>,
}

impl Card {
    pub fn power_toughness(&self) -> Option<String> {
        match (&self.power, &self.toughness) {
            (Some(p), Some(t)) => Some(format!("{p}/{t}")),
            _ => None,
        }
    }

    /// Get all oracle text, including from all faces for DFCs
    pub fn all_oracle_text(&self) -> Vec<&str> {
        let mut texts = Vec::new();

        // Add main oracle text if present
        if let Some(text) = &self.oracle_text {
            texts.push(text.as_str());
        }

        // Add oracle text from each face
        if let Some(faces) = &self.card_faces {
            for face in faces {
                if let Some(text) = &face.oracle_text {
                    texts.push(text.as_str());
                }
            }
        }

        texts
    }

    /// Get all type lines, including from all faces for DFCs
    pub fn all_type_lines(&self) -> Vec<&str> {
        let mut types = Vec::new();

        // Add main type line
        if !self.type_line.is_empty() {
            types.push(self.type_line.as_str());
        }

        // Add type lines from each face
        if let Some(faces) = &self.card_faces {
            for face in faces {
                if let Some(type_line) = &face.type_line {
                    types.push(type_line.as_str());
                }
            }
        }

        types
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prices {
    pub usd: Option<String>,
    pub usd_foil: Option<String>,
    pub eur: Option<String>,
    pub tix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUris {
    pub small: Option<String>,
    pub normal: Option<String>,
    pub large: Option<String>,
    pub png: Option<String>,
    pub art_crop: Option<String>,
    pub border_crop: Option<String>,
}
