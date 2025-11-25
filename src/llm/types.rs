/// Result of LLM analysis (shared across all providers)
#[derive(Debug, Clone)]
pub struct LlmAnalysisResult {
    pub full_response: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}
