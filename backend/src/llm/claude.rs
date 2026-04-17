use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::LlmProvider;

pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: String, max_tokens: u32, temperature: f32) -> Self {
        let model = if model.is_empty() { "claude-sonnet-4-20250514".into() } else { model };
        Self {
            client: Client::new(),
            api_key,
            model,
            max_tokens,
            temperature,
        }
    }
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    system: String,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: Option<String>,
}

#[derive(Serialize)]
struct EmbeddingFallbackRequest {
    model: String,
    input: String,
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn complete(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            system: system_prompt.to_string(),
            messages: vec![ClaudeMessage {
                role: "user".into(),
                content: user_message.into(),
            }],
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to call Claude API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Claude API error ({}): {}", status, body);
        }

        let body: ClaudeResponse = response.json().await.context("Failed to parse Claude response")?;
        let text = body.content.first()
            .and_then(|c| c.text.clone())
            .unwrap_or_default();

        Ok(text)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Claude doesn't have a native embedding API. Use Voyager via a proxy
        // or fall back to a simple hash-based embedding for development.
        // In production, pair with OpenAI embeddings or a local model.
        tracing::warn!("Claude provider does not support native embeddings. Using placeholder. Configure OpenAI or another embedding provider.");
        Ok(placeholder_embedding(text))
    }

    fn name(&self) -> &str { "claude" }
    fn model(&self) -> &str { &self.model }
}

/// Deterministic placeholder embedding for development/testing.
/// NOT for production — use a real embedding model.
fn placeholder_embedding(text: &str) -> Vec<f32> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hash = hasher.finalize();
    let mut embedding = vec![0.0f32; 1536];
    for (i, byte) in hash.iter().cycle().take(1536).enumerate() {
        embedding[i] = (*byte as f32 / 255.0) * 2.0 - 1.0;
    }
    // Normalize
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut embedding { *v /= norm; }
    }
    embedding
}
