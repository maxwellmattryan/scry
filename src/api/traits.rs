use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt;

use super::types::Card;

/// Error type for API operations
#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
    pub is_retryable: bool,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

impl ApiError {
    pub fn retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_retryable: true,
        }
    }

    pub fn not_retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_retryable: false,
        }
    }
}

/// Trait for MTG card API providers
#[async_trait]
pub trait CardApi: Send + Sync {
    /// Get the provider name (for logging/display)
    fn name(&self) -> &'static str;

    /// Search for a card by name (fuzzy matching where supported)
    async fn search_card(&self, query: &str) -> Result<Card, ApiError>;

    /// Get a card by its provider-specific ID
    async fn get_card_by_id(&self, id: &str) -> Result<Card, ApiError>;

    /// Batch fetch cards by name
    async fn batch_fetch_cards(
        &self,
        names: Vec<String>,
    ) -> Result<HashMap<String, Card>, ApiError>;
}
