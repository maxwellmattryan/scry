#![allow(dead_code)]

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cache::CardCache;

const SCRYFALL_API_BASE: &str = "https://api.scryfall.com";
const APP_USER_AGENT: &str = "mtg-cli/0.1.0";

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

    /// Check if this is a double-faced or multi-faced card
    pub fn is_multi_faced(&self) -> bool {
        self.card_faces
            .as_ref()
            .is_some_and(|faces| faces.len() > 1)
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

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    code: String,
    details: String,
}

pub struct ScryfallClient {
    client: reqwest::Client,
    cache: CardCache,
}

impl ScryfallClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(APP_USER_AGENT));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            cache: CardCache::new(),
        }
    }

    pub async fn search_card(&self, query: &str) -> Result<Card, String> {
        // Check cache first
        if let Some(card) = self.cache.get(query) {
            return Ok(card);
        }

        let url = format!(
            "{}/cards/named?fuzzy={}",
            SCRYFALL_API_BASE,
            urlencoding::encode(query)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let card: Card = response.json().await.map_err(|e| e.to_string())?;
            self.cache.set(&card.name, &card);
            Ok(card)
        } else {
            let error: ErrorResponse = response.json().await.map_err(|e| e.to_string())?;
            Err(format!("{}: {}", error.code, error.details))
        }
    }

    pub async fn get_card_by_id(&self, id: &str) -> Result<Card, String> {
        // Check cache first
        if let Some(card) = self.cache.get(id) {
            return Ok(card);
        }

        let url = format!("{SCRYFALL_API_BASE}/cards/{id}");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let card: Card = response.json().await.map_err(|e| e.to_string())?;
            self.cache.set(&card.id, &card);
            Ok(card)
        } else {
            let error: ErrorResponse = response.json().await.map_err(|e| e.to_string())?;
            Err(format!("{}: {}", error.code, error.details))
        }
    }
}

/// Identifier for batch card fetching
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum CardIdentifier {
    Name { name: String },
    Id { id: String },
}

/// Response from the /cards/collection endpoint
#[derive(Debug, Deserialize)]
struct CollectionResponse {
    data: Vec<Card>,
    not_found: Vec<serde_json::Value>,
}

impl ScryfallClient {
    /// Fetch multiple cards by name using the /cards/collection endpoint
    /// This is much faster than individual requests (up to 75 cards per request)
    pub async fn batch_fetch_cards(
        &self,
        names: Vec<String>,
    ) -> Result<HashMap<String, Card>, String> {
        let mut results = HashMap::new();
        let mut uncached_names = Vec::new();

        // Check cache first for all cards
        for name in &names {
            if let Some(card) = self.cache.get(name) {
                results.insert(name.clone(), card);
            } else {
                uncached_names.push(name.clone());
            }
        }

        // If all cards were cached, return early
        if uncached_names.is_empty() {
            return Ok(results);
        }

        // Batch fetch uncached cards (75 per request is Scryfall's limit)
        for chunk in uncached_names.chunks(75) {
            let identifiers: Vec<CardIdentifier> = chunk
                .iter()
                .map(|name| CardIdentifier::Name { name: name.clone() })
                .collect();

            let url = format!("{SCRYFALL_API_BASE}/cards/collection");

            let response = self
                .client
                .post(&url)
                .json(&serde_json::json!({ "identifiers": identifiers }))
                .send()
                .await
                .map_err(|e| format!("Batch fetch failed: {e}"))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(format!("Scryfall API error ({status}): {error_text}"));
            }

            let collection: CollectionResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {e}"))?;

            // Add fetched cards to results and cache
            for card in collection.data {
                self.cache.set(&card.name, &card);
                results.insert(card.name.clone(), card);
            }

            // Log not found cards (but don't fail)
            for not_found in &collection.not_found {
                if let Some(name) = not_found.get("name").and_then(|n| n.as_str()) {
                    eprintln!("Warning: Card not found: {name}");
                }
            }

            // Small delay between batch requests to be nice to the API
            if chunk.len() == 75 {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }

        Ok(results)
    }
}

impl Default for ScryfallClient {
    fn default() -> Self {
        Self::new()
    }
}
