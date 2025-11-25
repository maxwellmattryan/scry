use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use super::prompt::{build_synergy_prompt, SYSTEM_PROMPT};
use super::traits::{LlmClient, LlmError};
use super::types::LlmAnalysisResult;
use crate::input::DeckList;
use crate::synergy::SynergyMatrix;

const DEFAULT_OLLAMA_HOST: &str = "http://localhost:11434";
const DEFAULT_MODEL: &str = "llama3.2";

// Ollama-specific types

#[derive(Debug, Clone, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OllamaResponse {
    response: String,
    #[serde(default)]
    prompt_eval_count: u32,
    #[serde(default)]
    eval_count: u32,
}

/// Ollama API client for LLM-enhanced synergy analysis (local models)
pub struct OllamaClient {
    client: reqwest::Client,
    host: String,
    model: String,
}

impl OllamaClient {
    /// Create a new Ollama client
    ///
    /// Uses environment variables for configuration:
    /// - `OLLAMA_HOST`: API host (default: http://localhost:11434)
    /// - `OLLAMA_MODEL`: Model to use (default: llama3.2)
    pub fn new() -> Result<Self, LlmError> {
        let host = env::var("OLLAMA_HOST").unwrap_or_else(|_| DEFAULT_OLLAMA_HOST.to_string());
        let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        Ok(Self {
            client: reqwest::Client::new(),
            host,
            model,
        })
    }
}

#[async_trait]
impl LlmClient for OllamaClient {
    fn name(&self) -> &'static str {
        "Ollama (Local)"
    }

    async fn analyze_synergies(
        &self,
        deck: &DeckList,
        matrix: &SynergyMatrix,
        report: &str,
    ) -> Result<LlmAnalysisResult, LlmError> {
        let prompt = build_synergy_prompt(deck, matrix, report);

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            system: SYSTEM_PROMPT.to_string(),
            stream: false,
        };

        let url = format!("{}/api/generate", self.host);

        let response = self
            .client
            .post(&url)
            .timeout(Duration::from_secs(300)) // Longer timeout for local models
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::retryable("Request timed out. The local model is taking too long.")
                } else if e.is_connect() {
                    LlmError::not_retryable(format!(
                        "Failed to connect to Ollama at {}. Is Ollama running?",
                        self.host
                    ))
                } else {
                    LlmError::retryable(format!("HTTP request failed: {e}"))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::not_retryable(format!(
                "Ollama API error ({status}): {text}"
            )));
        }

        let api_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| LlmError::not_retryable(format!("Failed to parse response: {e}")))?;

        Ok(LlmAnalysisResult {
            full_response: api_response.response,
            input_tokens: api_response.prompt_eval_count,
            output_tokens: api_response.eval_count,
        })
    }
}
