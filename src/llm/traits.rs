use async_trait::async_trait;

use super::types::LlmAnalysisResult;
use crate::input::DeckList;
use crate::synergy::SynergyMatrix;

/// Error type for LLM operations
#[derive(Debug, Clone)]
pub struct LlmError {
    pub message: String,
    #[allow(dead_code)]
    pub is_retryable: bool,
}

impl LlmError {
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

    pub fn missing_api_key(provider: &str, env_var: &str) -> Self {
        Self::not_retryable(format!(
            "{provider} API key not set. Set {env_var} environment variable."
        ))
    }
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LlmError {}

/// Trait for LLM providers used in synergy analysis
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Get the provider name (for logging/display)
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Analyze deck synergies using the LLM
    async fn analyze_synergies(
        &self,
        deck: &DeckList,
        matrix: &SynergyMatrix,
        report: &str,
    ) -> Result<LlmAnalysisResult, LlmError>;
}
