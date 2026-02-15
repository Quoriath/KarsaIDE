use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::config_manager::AIConfig;
use tauri::{AppHandle, Emitter};
use futures::StreamExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Option<Message>,
    delta: Option<Delta>,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KiloModelsResponse {
    #[serde(default)]
    data: Vec<KiloModel>,
    #[serde(default)]
    models: Vec<KiloModel>,
}

#[derive(Debug, Deserialize)]
struct KiloModel {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    object: Option<String>,
    #[serde(default)]
    owned_by: Option<String>,
}

pub struct AIClient {
    client: Client,
}

impl AIClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch_kilo_models(&self, api_key: Option<String>) -> Result<Vec<ModelInfo>, String> {
        let mut req = self.client
            .get("https://api.kilo.ai/api/gateway/models");

        if let Some(key) = api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("Failed to fetch models: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        // Try to parse as KiloModelsResponse
        let text = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // Try parsing as object with data or models field
        let kilo_response: KiloModelsResponse = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Use data field if available, otherwise models field
        let models_list = if !kilo_response.data.is_empty() {
            kilo_response.data
        } else {
            kilo_response.models
        };

        let models = models_list
            .into_iter()
            .map(|m| {
                let display_name = m.name.clone()
                    .or_else(|| m.owned_by.clone())
                    .unwrap_or_else(|| m.id.clone());
                
                ModelInfo {
                    id: m.id.clone(),
                    name: display_name,
                    provider: "kilo".to_string(),
                    description: m.object.clone(),
                }
            })
            .collect();

        Ok(models)
    }

    pub async fn test_connection(&self, _config: &AIConfig) -> Result<String, String> {
        Ok("Connection successful".to_string())
    }

    pub async fn send_chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        config: &AIConfig,
    ) -> Result<String, String> {
        let request = ChatRequest {
            model: config.model_name.clone(),
            messages,
            stream: false,
        };

        let mut req = self.client
            .post(&config.base_url)
            .json(&request);

        if let Some(api_key) = &config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, body));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        chat_response
            .choices
            .first()
            .and_then(|c| c.message.as_ref())
            .map(|m| m.content.clone())
            .ok_or_else(|| "No response from AI".to_string())
    }

    pub async fn send_chat_completion_stream(
        &self,
        app: AppHandle,
        messages: Vec<ChatMessage>,
        config: &AIConfig,
    ) -> Result<(), String> {
        let request = ChatRequest {
            model: config.model_name.clone(),
            messages,
            stream: true,
        };

        let mut req = self.client
            .post(&config.base_url)
            .json(&request);

        if let Some(api_key) = &config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, body));
        }

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| format!("Stream error: {}", e))?;
            let text = String::from_utf8_lossy(&bytes);

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        let _ = app.emit("ai-stream-done", ());
                        break;
                    }

                    if let Ok(chunk_data) = serde_json::from_str::<ChatResponse>(data) {
                        if let Some(choice) = chunk_data.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    let _ = app.emit("ai-stream-chunk", content.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = app.emit("ai-stream-done", ());
        Ok(())
    }
}
