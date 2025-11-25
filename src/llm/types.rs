use serde::{Deserialize, Serialize};

/// Anthropic API message role
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum Role {
    User,
    Assistant,
}

/// A single message in the conversation
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Request body for Claude API
#[derive(Debug, Clone, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

/// Content block in response
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

/// Response from Claude API
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Error response from API
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicError {
    pub error: ErrorDetails,
}

/// Error details
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorDetails {
    pub message: String,
}

/// Result of LLM analysis
#[derive(Debug, Clone)]
pub struct LlmAnalysisResult {
    pub full_response: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}
