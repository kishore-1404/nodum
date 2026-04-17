use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Unified trait for all LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a chat completion request and get the response text
    async fn complete(&self, system_prompt: &str, user_message: &str) -> Result<String>;

    /// Generate embeddings for a text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Provider name for logging
    fn name(&self) -> &str;

    /// Model identifier
    fn model(&self) -> &str;
}

/// Configuration for creating a provider instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub model_name: String,
    pub base_url: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model_name: String::new(),
            base_url: None,
            max_tokens: 2048,
            temperature: 0.3,
        }
    }
}

/// Factory function to create a provider from config
pub fn create_provider(
    provider_type: &str,
    config: ProviderConfig,
) -> Result<Box<dyn LlmProvider>> {
    match provider_type {
        "claude" | "anthropic" => {
            let key = config.api_key.ok_or_else(|| anyhow::anyhow!("Claude API key required"))?;
            Ok(Box::new(super::claude::ClaudeProvider::new(key, config.model_name, config.max_tokens, config.temperature)))
        }
        "openai" | "gpt" => {
            let key = config.api_key.ok_or_else(|| anyhow::anyhow!("OpenAI API key required"))?;
            Ok(Box::new(super::openai::OpenAIProvider::new(key, config.model_name, config.base_url, config.max_tokens, config.temperature)))
        }
        "gemini" | "google" => {
            let key = config.api_key.ok_or_else(|| anyhow::anyhow!("Gemini API key required"))?;
            Ok(Box::new(super::gemini::GeminiProvider::new(key, config.model_name, config.max_tokens, config.temperature)))
        }
        "ollama" => {
            let base = config.base_url.unwrap_or_else(|| "http://localhost:11434".into());
            Ok(Box::new(super::openai::OpenAIProvider::new(
                "ollama".into(), config.model_name, Some(format!("{}/v1", base)),
                config.max_tokens, config.temperature,
            )))
        }
        _ => Err(anyhow::anyhow!("Unsupported provider: {}", provider_type)),
    }
}
