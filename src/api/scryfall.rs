use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cache::CardCache;

const SCRYFALL_API_BASE: &str = "https://api.scryfall.com";
const APP_USER_AGENT: &str = "mtg-cli/0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub mana_cost: Option<String>,
    pub cmc: f64,
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
}

impl Card {
    pub fn power_toughness(&self) -> Option<String> {
        match (&self.power, &self.toughness) {
            (Some(p), Some(t)) => Some(format!("{}/{}", p, t)),
            _ => None,
        }
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
struct SearchResponse {
    data: Vec<Card>,
    total_cards: u32,
    has_more: bool,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    status: u32,
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

        let url = format!("{}/cards/{}", SCRYFALL_API_BASE, id);

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

    pub async fn search_cards(&self, query: &str) -> Result<Vec<Card>, String> {
        let url = format!(
            "{}/cards/search?q={}",
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
            let search: SearchResponse = response.json().await.map_err(|e| e.to_string())?;
            Ok(search.data)
        } else {
            let error: ErrorResponse = response.json().await.map_err(|e| e.to_string())?;
            Err(format!("{}: {}", error.code, error.details))
        }
    }
}

impl Default for ScryfallClient {
    fn default() -> Self {
        Self::new()
    }
}
