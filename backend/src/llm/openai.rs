use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::LlmProvider;

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
    max_tokens: u32,
    temperature: f32,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String, base_url: Option<String>, max_tokens: u32, temperature: f32) -> Self {
        let model = if model.is_empty() { "gpt-4o".into() } else { model };
        let base_url = base_url.unwrap_or_else(|| "https://api.openai.com/v1".into());
        Self { client: Client::new(), api_key, model, base_url, max_tokens, temperature }
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<ChatMessage>,
    response_format: ResponseFormat,
}

#[derive(Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn complete(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        let request = ChatRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            messages: vec![
                ChatMessage { role: "system".into(), content: system_prompt.into() },
                ChatMessage { role: "user".into(), content: user_message.into() },
            ],
            response_format: ResponseFormat { r#type: "json_object".into() },
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to call OpenAI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, body);
        }

        let body: ChatResponse = response.json().await.context("Failed to parse OpenAI response")?;
        let text = body.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(text)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            model: "text-embedding-3-small".into(),
            input: text.into(),
        };

        let response = self.client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to call embedding API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Embedding API error ({}): {}", status, body);
        }

        let body: EmbeddingResponse = response.json().await?;
        let embedding = body.data.first()
            .map(|d| d.embedding.clone())
            .unwrap_or_default();

        Ok(embedding)
    }

    fn name(&self) -> &str { "openai" }
    fn model(&self) -> &str { &self.model }
}
