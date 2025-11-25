use super::anthropic::AnthropicClient;
use super::ollama::OllamaClient;
use super::openai::OpenAiClient;
use super::traits::{LlmClient, LlmError};

/// LLM provider selection
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LlmProvider {
    #[default]
    Anthropic,
    OpenAi,
    Ollama,
}

impl LlmProvider {
    /// Get the display name for this provider
    pub fn name(&self) -> &'static str {
        match self {
            LlmProvider::Anthropic => "Anthropic (Claude)",
            LlmProvider::OpenAi => "OpenAI (GPT)",
            LlmProvider::Ollama => "Ollama (Local)",
        }
    }

    /// Get the environment variable name for this provider's API key
    pub fn env_var(&self) -> &'static str {
        match self {
            LlmProvider::Anthropic => "ANTHROPIC_API_KEY",
            LlmProvider::OpenAi => "OPENAI_API_KEY",
            LlmProvider::Ollama => "OLLAMA_HOST",
        }
    }
}

/// Create an LlmClient based on provider selection
pub fn create_llm_client(provider: LlmProvider) -> Result<Box<dyn LlmClient>, LlmError> {
    match provider {
        LlmProvider::Anthropic => AnthropicClient::new().map(|c| Box::new(c) as Box<dyn LlmClient>),
        LlmProvider::OpenAi => OpenAiClient::new().map(|c| Box::new(c) as Box<dyn LlmClient>),
        LlmProvider::Ollama => OllamaClient::new().map(|c| Box::new(c) as Box<dyn LlmClient>),
    }
}
