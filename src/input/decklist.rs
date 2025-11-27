#![allow(dead_code)]

use crate::api::Card;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Represents a single entry in a decklist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckEntry {
    pub quantity: u32,
    pub card_name: String,
    pub card: Option<Card>,
    pub section: DeckSection,
}

/// The section of the deck a card belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DeckSection {
    Commander,
    #[default]
    Mainboard,
    Sideboard,
    Maybeboard,
}

/// The source of a decklist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeckSource {
    TextFile(String),
    Moxfield(String),
    Manual,
}

/// A complete decklist with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckList {
    pub name: Option<String>,
    pub format: Option<String>,
    pub entries: Vec<DeckEntry>,
    pub source: DeckSource,
    /// Indicates the decklist intentionally excludes basic lands (common when exporting from Moxfield)
    pub excludes_lands: bool,
}

impl DeckList {
    /// Create a new empty decklist
    pub fn new(source: DeckSource) -> Self {
        Self {
            name: None,
            format: None,
            entries: Vec::new(),
            source,
            excludes_lands: false,
        }
    }

    /// Get all entries that have been hydrated with card data
    pub fn cards(&self) -> impl Iterator<Item = &DeckEntry> {
        self.entries.iter().filter(|e| e.card.is_some())
    }

    /// Get mainboard entries (including commander)
    pub fn mainboard(&self) -> impl Iterator<Item = &DeckEntry> {
        self.entries
            .iter()
            .filter(|e| e.section == DeckSection::Mainboard || e.section == DeckSection::Commander)
    }

    /// Get sideboard entries
    pub fn sideboard(&self) -> impl Iterator<Item = &DeckEntry> {
        self.entries
            .iter()
            .filter(|e| e.section == DeckSection::Sideboard)
    }

    /// Get commander entries
    pub fn commanders(&self) -> impl Iterator<Item = &DeckEntry> {
        self.entries
            .iter()
            .filter(|e| e.section == DeckSection::Commander)
    }

    /// Get total card count (sum of quantities)
    pub fn total_cards(&self) -> u32 {
        self.entries.iter().map(|e| e.quantity).sum()
    }

    /// Get the count of unique cards
    pub fn unique_cards(&self) -> usize {
        self.entries.len()
    }

    /// Get card names for hydration (deduplicated)
    pub fn card_names(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.card_name.clone()).collect()
    }

    /// Add an entry to the decklist
    pub fn add_entry(&mut self, quantity: u32, card_name: String, section: DeckSection) {
        self.entries.push(DeckEntry {
            quantity,
            card_name,
            card: None,
            section,
        });
    }

    /// Count total lands in the mainboard (from hydrated card data)
    pub fn count_lands(&self) -> u32 {
        self.mainboard()
            .filter(|e| {
                e.card
                    .as_ref()
                    .is_some_and(|c| c.type_line.to_lowercase().contains("land"))
            })
            .map(|e| e.quantity)
            .sum()
    }
}

/// Trait for parsing decklists from various sources
#[async_trait]
pub trait DeckListParser: Send + Sync {
    async fn parse(&self, source: &str) -> Result<DeckList, String>;
}
