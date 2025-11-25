#![allow(dead_code)]

use async_trait::async_trait;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::Deserialize;
use std::collections::HashMap;

use super::decklist::{DeckEntry, DeckList, DeckListParser, DeckSection, DeckSource};

const MOXFIELD_API_BASE: &str = "https://api2.moxfield.com/v2";
const APP_USER_AGENT: &str = "mtg-cli/0.1.0";

/// Response structure for Moxfield deck API
#[derive(Debug, Deserialize)]
struct MoxfieldDeckResponse {
    name: String,
    format: Option<String>,
    mainboard: HashMap<String, MoxfieldCardEntry>,
    sideboard: HashMap<String, MoxfieldCardEntry>,
    commanders: HashMap<String, MoxfieldCardEntry>,
    #[serde(default)]
    companions: HashMap<String, MoxfieldCardEntry>,
    #[serde(default)]
    maybeboard: HashMap<String, MoxfieldCardEntry>,
}

#[derive(Debug, Deserialize)]
struct MoxfieldCardEntry {
    quantity: u32,
    card: MoxfieldCardData,
}

#[derive(Debug, Deserialize)]
struct MoxfieldCardData {
    name: String,
    #[serde(default)]
    scryfall_id: Option<String>,
}

/// Client for fetching decklists from Moxfield
pub struct MoxfieldClient {
    client: reqwest::Client,
}

impl MoxfieldClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(APP_USER_AGENT));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }

    /// Extract deck ID from a Moxfield URL
    /// Supports:
    /// - https://www.moxfield.com/decks/abc123
    /// - https://moxfield.com/decks/abc123
    /// - moxfield.com/decks/abc123
    /// - abc123 (just the ID)
    pub fn extract_deck_id(url: &str) -> Option<String> {
        // If it's just an ID (alphanumeric with some special chars)
        let id_only = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
        if id_only.is_match(url) {
            return Some(url.to_string());
        }

        // Extract from URL
        let url_pattern = Regex::new(r"moxfield\.com/decks/([a-zA-Z0-9_-]+)").unwrap();
        url_pattern
            .captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Fetch a deck by its ID
    pub async fn fetch_deck(&self, deck_id: &str) -> Result<DeckList, String> {
        let url = format!("{MOXFIELD_API_BASE}/decks/all/{deck_id}");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch from Moxfield: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Moxfield API error ({status}): {error_text}"));
        }

        let moxfield_deck: MoxfieldDeckResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Moxfield response: {e}"))?;

        Ok(self.convert_to_decklist(moxfield_deck, deck_id))
    }

    /// Convert Moxfield response to our DeckList format
    fn convert_to_decklist(&self, moxfield: MoxfieldDeckResponse, deck_id: &str) -> DeckList {
        let mut deck_list = DeckList::new(DeckSource::Moxfield(deck_id.to_string()));
        deck_list.name = Some(moxfield.name);
        deck_list.format = moxfield.format;

        // Add commanders
        for (_, entry) in moxfield.commanders {
            deck_list.entries.push(DeckEntry {
                quantity: entry.quantity,
                card_name: entry.card.name,
                card: None,
                section: DeckSection::Commander,
            });
        }

        // Add companions as commanders (they're special too)
        for (_, entry) in moxfield.companions {
            deck_list.entries.push(DeckEntry {
                quantity: entry.quantity,
                card_name: entry.card.name,
                card: None,
                section: DeckSection::Commander,
            });
        }

        // Add mainboard
        for (_, entry) in moxfield.mainboard {
            deck_list.entries.push(DeckEntry {
                quantity: entry.quantity,
                card_name: entry.card.name,
                card: None,
                section: DeckSection::Mainboard,
            });
        }

        // Add sideboard
        for (_, entry) in moxfield.sideboard {
            deck_list.entries.push(DeckEntry {
                quantity: entry.quantity,
                card_name: entry.card.name,
                card: None,
                section: DeckSection::Sideboard,
            });
        }

        // Add maybeboard
        for (_, entry) in moxfield.maybeboard {
            deck_list.entries.push(DeckEntry {
                quantity: entry.quantity,
                card_name: entry.card.name,
                card: None,
                section: DeckSection::Maybeboard,
            });
        }

        deck_list
    }

    /// Check if a string looks like a Moxfield URL or ID
    pub fn is_moxfield_source(source: &str) -> bool {
        source.contains("moxfield.com") || Self::extract_deck_id(source).is_some()
    }
}

impl Default for MoxfieldClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DeckListParser for MoxfieldClient {
    async fn parse(&self, source: &str) -> Result<DeckList, String> {
        let deck_id = Self::extract_deck_id(source)
            .ok_or_else(|| format!("Invalid Moxfield URL or deck ID: {source}"))?;

        self.fetch_deck(&deck_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_deck_id_full_url() {
        let url = "https://www.moxfield.com/decks/abc123";
        assert_eq!(
            MoxfieldClient::extract_deck_id(url),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_extract_deck_id_no_www() {
        let url = "https://moxfield.com/decks/abc123";
        assert_eq!(
            MoxfieldClient::extract_deck_id(url),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_extract_deck_id_just_id() {
        let id = "abc123";
        assert_eq!(
            MoxfieldClient::extract_deck_id(id),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_extract_deck_id_complex_id() {
        let url = "https://www.moxfield.com/decks/aBc_123-xyz";
        assert_eq!(
            MoxfieldClient::extract_deck_id(url),
            Some("aBc_123-xyz".to_string())
        );
    }
}
