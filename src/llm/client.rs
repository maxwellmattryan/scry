use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::env;
use std::time::Duration;

use super::prompt::{build_synergy_prompt, SYSTEM_PROMPT};
use super::types::*;
use crate::input::DeckList;
use crate::synergy::SynergyMatrix;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const MAX_TOKENS: u32 = 4096;

/// Anthropic API client for LLM-enhanced synergy analysis
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicClient {
    /// Create a new Anthropic client from environment variable
    pub fn new() -> Result<Self, String> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }

    /// Analyze deck synergies using Claude
    pub async fn analyze_synergies(
        &self,
        deck: &DeckList,
        matrix: &SynergyMatrix,
        report: &str,
    ) -> Result<LlmAnalysisResult, String> {
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
            HeaderValue::from_str(&self.api_key).map_err(|_| "Invalid API key format")?,
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
                    "Request timed out. The LLM is taking too long.".to_string()
                } else if e.is_connect() {
                    "Failed to connect to Anthropic API.".to_string()
                } else {
                    format!("HTTP request failed: {e}")
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            return match status.as_u16() {
                401 => Err("Invalid API key. Check ANTHROPIC_API_KEY.".to_string()),
                429 => Err("Rate limited. Please wait and try again.".to_string()),
                _ => {
                    let error: AnthropicError = response
                        .json()
                        .await
                        .map_err(|e| format!("Failed to parse error: {e}"))?;
                    Err(format!("API error: {}", error.error.message))
                }
            };
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;

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
