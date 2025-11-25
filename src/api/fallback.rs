use async_trait::async_trait;
use std::collections::HashMap;

use super::traits::{ApiError, CardApi};
use super::types::Card;

/// A client that wraps multiple API providers and falls back on failure
pub struct FallbackClient {
    providers: Vec<Box<dyn CardApi>>,
}

impl FallbackClient {
    /// Create with primary and fallback providers
    pub fn with_fallback(primary: Box<dyn CardApi>, fallback: Box<dyn CardApi>) -> Self {
        Self {
            providers: vec![primary, fallback],
        }
    }
}

#[async_trait]
impl CardApi for FallbackClient {
    fn name(&self) -> &'static str {
        "Fallback"
    }

    async fn search_card(&self, query: &str) -> Result<Card, ApiError> {
        let mut last_error = None;

        for provider in &self.providers {
            match provider.search_card(query).await {
                Ok(card) => return Ok(card),
                Err(e) => {
                    eprintln!(
                        "Warning: {} failed for '{}': {}",
                        provider.name(),
                        query,
                        e.message
                    );
                    last_error = Some(e);
                    // Try next provider regardless of error type for searches
                    // (card might exist in one database but not another)
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ApiError::not_retryable("All providers failed")))
    }

    async fn get_card_by_id(&self, id: &str) -> Result<Card, ApiError> {
        // ID lookups only make sense for the primary provider
        // since IDs are provider-specific
        self.providers.first().unwrap().get_card_by_id(id).await
    }

    async fn batch_fetch_cards(
        &self,
        names: Vec<String>,
    ) -> Result<HashMap<String, Card>, ApiError> {
        let mut last_error = None;

        for provider in &self.providers {
            match provider.batch_fetch_cards(names.clone()).await {
                Ok(cards) => return Ok(cards),
                Err(e) => {
                    eprintln!(
                        "Warning: {} batch fetch failed: {}",
                        provider.name(),
                        e.message
                    );
                    last_error = Some(e);
                    // Try next provider
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ApiError::not_retryable("All providers failed")))
    }
}
