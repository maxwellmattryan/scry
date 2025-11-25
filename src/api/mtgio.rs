use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::Deserialize;
use std::collections::HashMap;

use super::cache::CardCache;
use super::traits::{ApiError, CardApi};
use super::types::{Card, ImageUris};

const MTGIO_API_BASE: &str = "https://api.magicthegathering.io/v1";
const APP_USER_AGENT: &str = "mtg-cli/0.1.0";

/// Response struct for MTG.io card format
#[derive(Debug, Deserialize)]
struct MtgIoCard {
    id: String,
    name: String,
    #[serde(rename = "manaCost")]
    mana_cost: Option<String>,
    cmc: Option<f64>,
    #[serde(rename = "type")]
    type_line: Option<String>,
    text: Option<String>, // MTG.io uses "text" not "oracle_text"
    power: Option<String>,
    toughness: Option<String>,
    colors: Option<Vec<String>>,
    #[serde(rename = "colorIdentity")]
    color_identity: Option<Vec<String>>,
    set: Option<String>,
    #[serde(rename = "setName")]
    set_name: Option<String>,
    rarity: Option<String>,
    #[serde(rename = "imageUrl")]
    image_url: Option<String>,
    legalities: Option<Vec<MtgIoLegality>>,
    #[serde(rename = "multiverseid")]
    #[allow(dead_code)]
    multiverse_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MtgIoLegality {
    format: String,
    legality: String,
}

#[derive(Debug, Deserialize)]
struct MtgIoResponse {
    cards: Vec<MtgIoCard>,
}

#[derive(Debug, Deserialize)]
struct MtgIoSingleResponse {
    card: MtgIoCard,
}

impl MtgIoCard {
    /// Convert MTG.io card format to unified Card type
    fn into_card(self) -> Card {
        // Convert legalities from array to HashMap
        let legalities: HashMap<String, String> = self
            .legalities
            .unwrap_or_default()
            .into_iter()
            .map(|l| (l.format.to_lowercase(), l.legality.to_lowercase()))
            .collect();

        Card {
            id: self.id,
            name: self.name.clone(),
            mana_cost: self.mana_cost,
            cmc: self.cmc.unwrap_or(0.0),
            type_line: self.type_line.unwrap_or_default(),
            oracle_text: self.text,
            power: self.power,
            toughness: self.toughness,
            colors: self.colors,
            color_identity: self.color_identity.unwrap_or_default(),
            set: self.set.unwrap_or_else(|| "???".to_string()),
            set_name: self.set_name.unwrap_or_else(|| "Unknown Set".to_string()),
            rarity: self.rarity.unwrap_or_else(|| "unknown".to_string()),
            prices: None, // MTG.io does not provide prices
            legalities,
            image_uris: self.image_url.map(|url| ImageUris {
                small: Some(url.clone()),
                normal: Some(url.clone()),
                large: Some(url),
                png: None,
                art_crop: None,
                border_crop: None,
            }),
            scryfall_uri: format!(
                "https://scryfall.com/search?q={}",
                urlencoding::encode(&self.name)
            ),
            card_faces: None, // MTG.io handles DFCs differently
            layout: None,
        }
    }
}

pub struct MtgIoClient {
    client: reqwest::Client,
    cache: CardCache,
}

impl MtgIoClient {
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
}

#[async_trait]
impl CardApi for MtgIoClient {
    fn name(&self) -> &'static str {
        "MTG.io"
    }

    async fn search_card(&self, query: &str) -> Result<Card, ApiError> {
        // Check cache first
        if let Some(card) = self.cache.get(query) {
            return Ok(card);
        }

        // MTG.io uses exact name matching with quotes for exact search
        let url = format!(
            "{}/cards?name=\"{}\"",
            MTGIO_API_BASE,
            urlencoding::encode(query)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::retryable(e.to_string()))?;

        if !response.status().is_success() {
            let is_server_error = response.status().is_server_error();
            return Err(if is_server_error {
                ApiError::retryable(format!("MTG.io API error: {}", response.status()))
            } else {
                ApiError::not_retryable(format!("MTG.io API error: {}", response.status()))
            });
        }

        let mtgio_response: MtgIoResponse = response
            .json()
            .await
            .map_err(|e| ApiError::not_retryable(format!("Failed to parse response: {e}")))?;

        if mtgio_response.cards.is_empty() {
            // Try without quotes for fuzzy-ish matching
            let url = format!(
                "{}/cards?name={}",
                MTGIO_API_BASE,
                urlencoding::encode(query)
            );

            let response = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| ApiError::retryable(e.to_string()))?;

            if !response.status().is_success() {
                return Err(ApiError::not_retryable(format!("Card not found: {query}")));
            }

            let mtgio_response: MtgIoResponse = response
                .json()
                .await
                .map_err(|e| ApiError::not_retryable(format!("Failed to parse response: {e}")))?;

            if mtgio_response.cards.is_empty() {
                return Err(ApiError::not_retryable(format!("Card not found: {query}")));
            }

            let card = mtgio_response.cards.into_iter().next().unwrap().into_card();
            self.cache.set(&card.name, &card);
            return Ok(card);
        }

        let card = mtgio_response.cards.into_iter().next().unwrap().into_card();
        self.cache.set(&card.name, &card);
        Ok(card)
    }

    async fn get_card_by_id(&self, id: &str) -> Result<Card, ApiError> {
        // Check cache first
        if let Some(card) = self.cache.get(id) {
            return Ok(card);
        }

        let url = format!("{MTGIO_API_BASE}/cards/{id}");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::retryable(e.to_string()))?;

        if !response.status().is_success() {
            let is_server_error = response.status().is_server_error();
            return Err(if is_server_error {
                ApiError::retryable(format!("MTG.io API error: {}", response.status()))
            } else {
                ApiError::not_retryable(format!("MTG.io API error: {}", response.status()))
            });
        }

        let single_response: MtgIoSingleResponse = response
            .json()
            .await
            .map_err(|e| ApiError::not_retryable(format!("Failed to parse response: {e}")))?;

        let card = single_response.card.into_card();
        self.cache.set(&card.id, &card);
        Ok(card)
    }

    async fn batch_fetch_cards(
        &self,
        names: Vec<String>,
    ) -> Result<HashMap<String, Card>, ApiError> {
        // MTG.io does not have a batch endpoint like Scryfall
        // We need to make individual requests (with rate limiting)
        let mut results = HashMap::new();

        for name in names {
            // Check cache first
            if let Some(card) = self.cache.get(&name) {
                results.insert(card.name.clone(), card);
                continue;
            }

            match self.search_card(&name).await {
                Ok(card) => {
                    results.insert(card.name.clone(), card);
                }
                Err(e) if !e.is_retryable => {
                    // Card not found, continue
                    eprintln!("Warning: Card not found: {name}");
                }
                Err(e) => {
                    // Retryable error, propagate
                    return Err(e);
                }
            }

            // Rate limit: MTG.io recommends delays between requests
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(results)
    }
}

impl Default for MtgIoClient {
    fn default() -> Self {
        Self::new()
    }
}
