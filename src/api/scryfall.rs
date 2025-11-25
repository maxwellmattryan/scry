#![allow(dead_code)]

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cache::CardCache;
use super::traits::{ApiError, CardApi};
use super::types::Card;

const SCRYFALL_API_BASE: &str = "https://api.scryfall.com";
const APP_USER_AGENT: &str = "mtg-cli/0.1.0";

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

    async fn search_card_impl(&self, query: &str) -> Result<Card, ApiError> {
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
            .map_err(|e| ApiError::retryable(e.to_string()))?;

        if response.status().is_success() {
            let card: Card = response
                .json()
                .await
                .map_err(|e| ApiError::not_retryable(format!("Failed to parse response: {e}")))?;
            self.cache.set(&card.name, &card);
            Ok(card)
        } else {
            let error: ErrorResponse = response
                .json()
                .await
                .map_err(|e| ApiError::retryable(e.to_string()))?;
            Err(ApiError::not_retryable(format!(
                "{}: {}",
                error.code, error.details
            )))
        }
    }

    async fn get_card_by_id_impl(&self, id: &str) -> Result<Card, ApiError> {
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
            .map_err(|e| ApiError::retryable(e.to_string()))?;

        if response.status().is_success() {
            let card: Card = response
                .json()
                .await
                .map_err(|e| ApiError::not_retryable(format!("Failed to parse response: {e}")))?;
            self.cache.set(&card.id, &card);
            Ok(card)
        } else {
            let error: ErrorResponse = response
                .json()
                .await
                .map_err(|e| ApiError::retryable(e.to_string()))?;
            Err(ApiError::not_retryable(format!(
                "{}: {}",
                error.code, error.details
            )))
        }
    }

    async fn batch_fetch_cards_impl(
        &self,
        names: Vec<String>,
    ) -> Result<HashMap<String, Card>, ApiError> {
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
                .map_err(|e| ApiError::retryable(format!("Batch fetch failed: {e}")))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(ApiError::retryable(format!(
                    "Scryfall API error ({status}): {error_text}"
                )));
            }

            let collection: CollectionResponse = response
                .json()
                .await
                .map_err(|e| ApiError::not_retryable(format!("Failed to parse response: {e}")))?;

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

#[async_trait]
impl CardApi for ScryfallClient {
    fn name(&self) -> &'static str {
        "Scryfall"
    }

    async fn search_card(&self, query: &str) -> Result<Card, ApiError> {
        self.search_card_impl(query).await
    }

    async fn get_card_by_id(&self, id: &str) -> Result<Card, ApiError> {
        self.get_card_by_id_impl(id).await
    }

    async fn batch_fetch_cards(
        &self,
        names: Vec<String>,
    ) -> Result<HashMap<String, Card>, ApiError> {
        self.batch_fetch_cards_impl(names).await
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

impl Default for ScryfallClient {
    fn default() -> Self {
        Self::new()
    }
}
