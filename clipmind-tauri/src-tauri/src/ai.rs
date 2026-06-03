use crate::types::ClipType;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MODEL: &str = "meta-llama/llama-3.1-8b-instruct:free";

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

pub struct AiClient {
    api_key: String,
    client: reqwest::Client,
}

impl AiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap(),
        }
    }

    pub async fn categorize(&self, content: &str) -> Result<ClipType> {
        let truncated = if content.len() > 500 {
            &content[..500]
        } else {
            content
        };
        let prompt = format!(
            "Categorize this clipboard text into ONE of: text, code, link. Respond with only the word.\n\nText: \"{}\"",
            truncated.replace('"', "'")
        );
        let req = ChatRequest {
            model: MODEL.to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a classifier. Reply with exactly one word: text, code, or link."
                        .to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            max_tokens: 5,
            temperature: 0.0,
        };
        let resp = self
            .client
            .post(OPENROUTER_URL)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await
            .context("AI request failed")?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("AI error: HTTP {}", resp.status()));
        }
        let body: ChatResponse = resp.json().await.context("AI parse failed")?;
        let text = body
            .choices
            .first()
            .map(|c| c.message.content.trim().to_lowercase())
            .unwrap_or_default();
        Ok(match text.as_str() {
            "code" => ClipType::Code,
            "link" => ClipType::Link,
            _ => ClipType::Text,
        })
    }
}
