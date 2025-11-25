use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use super::prompt::{build_synergy_prompt, SYSTEM_PROMPT};
use super::traits::{LlmClient, LlmError};
use super::types::LlmAnalysisResult;
use crate::input::DeckList;
use crate::synergy::SynergyMatrix;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const MAX_TOKENS: u32 = 4096;

// Anthropic-specific types

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicError {
    error: ErrorDetails,
}

#[derive(Debug, Clone, Deserialize)]
struct ErrorDetails {
    message: String,
}

/// Anthropic API client for LLM-enhanced synergy analysis
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicClient {
    /// Create a new Anthropic client from environment variable
    pub fn new() -> Result<Self, LlmError> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| LlmError::missing_api_key("Anthropic", "ANTHROPIC_API_KEY"))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    fn name(&self) -> &'static str {
        "Anthropic (Claude)"
    }

    async fn analyze_synergies(
        &self,
        deck: &DeckList,
        matrix: &SynergyMatrix,
        report: &str,
    ) -> Result<LlmAnalysisResult, LlmError> {
        let prompt = build_synergy_prompt(deck, matrix, report);

        let request = AnthropicRequest {
            model: DEFAULT_MODEL.to_string(),
            max_tokens: MAX_TOKENS,
            messages: vec![Message {
                role: Role::User,
                content: prompt,
            }],
            system: Some(SYSTEM_PROMPT.to_string()),
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key)
                .map_err(|_| LlmError::not_retryable("Invalid API key format"))?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .headers(headers)
            .timeout(Duration::from_secs(120))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::retryable("Request timed out. The LLM is taking too long.")
                } else if e.is_connect() {
                    LlmError::retryable("Failed to connect to Anthropic API.")
                } else {
                    LlmError::retryable(format!("HTTP request failed: {e}"))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            return match status.as_u16() {
                401 => Err(LlmError::not_retryable(
                    "Invalid API key. Check ANTHROPIC_API_KEY.",
                )),
                429 => Err(LlmError::retryable(
                    "Rate limited. Please wait and try again.",
                )),
                _ => {
                    let error: AnthropicError = response.json().await.map_err(|e| {
                        LlmError::not_retryable(format!("Failed to parse error: {e}"))
                    })?;
                    Err(LlmError::not_retryable(format!(
                        "API error: {}",
                        error.error.message
                    )))
                }
            };
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LlmError::not_retryable(format!("Failed to parse response: {e}")))?;

        let full_response = api_response
            .content
            .iter()
            .map(|block| match block {
                ContentBlock::Text { text } => text.as_str(),
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(LlmAnalysisResult {
            full_response,
            input_tokens: api_response.usage.input_tokens,
            output_tokens: api_response.usage.output_tokens,
        })
    }
}
