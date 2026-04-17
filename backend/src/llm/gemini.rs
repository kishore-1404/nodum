use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::LlmProvider;

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String, max_tokens: u32, temperature: f32) -> Self {
        let model = if model.is_empty() { "gemini-2.0-flash".into() } else { model };
        Self { client: Client::new(), api_key, model, max_tokens, temperature }
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    system_instruction: GeminiContent,
    generation_config: GenerationConfig,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    max_output_tokens: u32,
    temperature: f32,
    response_mime_type: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: GeminiContent,
}

#[derive(Serialize)]
struct GeminiEmbedRequest {
    model: String,
    content: GeminiContent,
}

#[derive(Deserialize)]
struct GeminiEmbedResponse {
    embedding: Option<GeminiEmbedding>,
}

#[derive(Deserialize)]
struct GeminiEmbedding {
    values: Vec<f32>,
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn complete(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: user_message.into() }],
                role: Some("user".into()),
            }],
            system_instruction: GeminiContent {
                parts: vec![GeminiPart { text: system_prompt.into() }],
                role: None,
            },
            generation_config: GenerationConfig {
                max_output_tokens: self.max_tokens,
                temperature: self.temperature,
                response_mime_type: "application/json".into(),
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to call Gemini API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API error ({}): {}", status, body);
        }

        let body: GeminiResponse = response.json().await.context("Failed to parse Gemini response")?;
        let text = body.candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .unwrap_or_default();

        Ok(text)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = GeminiEmbedRequest {
            model: format!("models/text-embedding-004"),
            content: GeminiContent {
                parts: vec![GeminiPart { text: text.into() }],
                role: None,
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent?key={}",
            self.api_key
        );

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to call Gemini embedding API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini embedding error ({}): {}", status, body);
        }

        let body: GeminiEmbedResponse = response.json().await?;
        let mut values = body.embedding.map(|e| e.values).unwrap_or_default();

        // Gemini embeddings are 768-dim; pad to 1536 for pgvector compatibility
        values.resize(1536, 0.0);
        Ok(values)
    }

    fn name(&self) -> &str { "gemini" }
    fn model(&self) -> &str { &self.model }
}
