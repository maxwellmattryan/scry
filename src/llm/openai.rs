use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use super::prompt::{build_synergy_prompt, SYSTEM_PROMPT};
use super::traits::{LlmClient, LlmError};
use super::types::LlmAnalysisResult;
use crate::input::DeckList;
use crate::synergy::SynergyMatrix;

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-4o";
const MAX_TOKENS: u32 = 4096;

// OpenAI-specific types

#[derive(Debug, Clone, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiResponse {
    choices: Vec<Choice>,
    usage: OpenAiUsage,
}

#[derive(Debug, Clone, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiError {
    error: OpenAiErrorDetails,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiErrorDetails {
    message: String,
}

/// OpenAI API client for LLM-enhanced synergy analysis
pub struct OpenAiClient {
    client: reqwest::Client,
    api_key: String,
}

impl OpenAiClient {
    /// Create a new OpenAI client from environment variable
    pub fn new() -> Result<Self, LlmError> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| LlmError::missing_api_key("OpenAI", "OPENAI_API_KEY"))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    fn name(&self) -> &'static str {
        "OpenAI (GPT)"
    }

    async fn analyze_synergies(
        &self,
        deck: &DeckList,
        matrix: &SynergyMatrix,
        report: &str,
    ) -> Result<LlmAnalysisResult, LlmError> {
        let prompt = build_synergy_prompt(deck, matrix, report);

        let request = OpenAiRequest {
            model: DEFAULT_MODEL.to_string(),
            max_tokens: MAX_TOKENS,
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: SYSTEM_PROMPT.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|_| LlmError::not_retryable("Invalid API key format"))?,
        );

        let response = self
            .client
            .post(OPENAI_API_URL)
            .headers(headers)
            .timeout(Duration::from_secs(120))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::retryable("Request timed out. The LLM is taking too long.")
                } else if e.is_connect() {
                    LlmError::retryable("Failed to connect to OpenAI API.")
                } else {
                    LlmError::retryable(format!("HTTP request failed: {e}"))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            // Try to parse the error body for all error cases
            let error_msg = response
                .json::<OpenAiError>()
                .await
                .map(|e| e.error.message)
                .unwrap_or_else(|_| "Unknown error".to_string());

            return match status.as_u16() {
                401 => Err(LlmError::not_retryable(format!(
                    "Invalid API key: {error_msg}"
                ))),
                429 => Err(LlmError::not_retryable(format!(
                    "Rate limited or quota exceeded: {error_msg}"
                ))),
                _ => Err(LlmError::not_retryable(format!("API error: {error_msg}"))),
            };
        }

        let api_response: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| LlmError::not_retryable(format!("Failed to parse response: {e}")))?;

        let full_response = api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(LlmAnalysisResult {
            full_response,
            input_tokens: api_response.usage.prompt_tokens,
            output_tokens: api_response.usage.completion_tokens,
        })
    }
}
