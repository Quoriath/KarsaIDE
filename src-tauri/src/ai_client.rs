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
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Option<Message>,
    pub delta: Option<Delta>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
    pub reasoning: Option<String>,
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
    pub client: Client,
}

impl AIClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .connect_timeout(std::time::Duration::from_secs(10))
                .pool_max_idle_per_host(5)
                .build()
                .unwrap(),
        }
    }

    pub async fn fetch_kilo_models(&self, api_key: Option<String>, force_refresh: bool) -> Result<Vec<ModelInfo>, String> {
        // Try cache first if not forcing refresh
        if !force_refresh {
            if let Some(cached) = crate::cache::Cache::get_models() {
                // Return cached immediately, spawn background refresh
                let api_key_clone = api_key.clone();
                let client = self.client.clone();
                tokio::spawn(async move {
                    if let Ok(fresh_models) = Self::fetch_models_from_api(&client, api_key_clone).await {
                        let _ = crate::cache::Cache::set_models(fresh_models);
                    }
                });
                return Ok(cached);
            }
        }
        
        // No cache or force refresh - fetch from API
        let models = Self::fetch_models_from_api(&self.client, api_key).await?;
        
        // Cache the result
        let _ = crate::cache::Cache::set_models(models.clone());
        
        Ok(models)
    }
    
    async fn fetch_models_from_api(client: &Client, api_key: Option<String>) -> Result<Vec<ModelInfo>, String> {
        let mut req = client.get("https://api.kilo.ai/api/gateway/models");

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

        let text = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let kilo_response: KiloModelsResponse = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

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
        let mut attempts = 0;
        let max_attempts = 3;
        
        loop {
            attempts += 1;
            
            let request = ChatRequest {
                model: config.model_name.clone(),
                messages: messages.clone(),
                stream: false,
            };

            let mut req = self.client
                .post(&config.base_url)
                .json(&request);

            if let Some(api_key) = &config.api_key {
                req = req.header("Authorization", format!("Bearer {}", api_key));
            }

            match req.send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        
                        // Don't retry on client errors (4xx)
                        if status.is_client_error() {
                            return Err(format!("API error {}: {}", status, body));
                        }
                        
                        // Retry on server errors (5xx)
                        if attempts >= max_attempts {
                            return Err(format!("API error {} after {} attempts: {}", status, attempts, body));
                        }
                        
                        tokio::time::sleep(std::time::Duration::from_millis(500 * attempts as u64)).await;
                        continue;
                    }

                    let chat_response: ChatResponse = response
                        .json()
                        .await
                        .map_err(|e| format!("Failed to parse response: {}", e))?;

                    let content = chat_response
                        .choices
                        .first()
                        .and_then(|c| c.message.as_ref())
                        .map(|m| m.content.clone())
                        .ok_or_else(|| "No response content".to_string())?;
                    
                    if content.trim().is_empty() {
                        return Err("AI returned empty response".to_string());
                    }
                    
                    return Ok(content);
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(format!("Request failed after {} attempts: {}", attempts, e));
                    }
                    
                    tokio::time::sleep(std::time::Duration::from_millis(500 * attempts as u64)).await;
                }
            }
        }
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
        let mut done_emitted = false;

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| format!("Stream error: {}", e))?;
            let text = String::from_utf8_lossy(&bytes);

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        if !done_emitted {
                            let _ = app.emit("ai-stream-done", ());
                            done_emitted = true;
                        }
                        break;
                    }

                    if let Ok(chunk_data) = serde_json::from_str::<ChatResponse>(data) {
                        if let Some(choice) = chunk_data.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(reasoning) = &delta.reasoning {
                                    if !reasoning.is_empty() {
                                        let _ = app.emit("ai-stream-reasoning", reasoning.clone());
                                    }
                                }
                                if let Some(content) = &delta.content {
                                    if !content.is_empty() {
                                        let _ = app.emit("ai-stream-chunk", content.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !done_emitted {
            let _ = app.emit("ai-stream-done", ());
        }
        Ok(())
    }
}
