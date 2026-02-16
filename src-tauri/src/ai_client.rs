use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::config_manager::AIConfig;
use tauri::{AppHandle, Emitter};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ChatResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub model: Option<String>,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Choice {
    pub index: Option<u32>,
    pub message: Option<Message>,
    pub delta: Option<Delta>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Message {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub reasoning: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
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

/// Stream state for cancellation
pub struct StreamState {
    pub cancelled: bool,
}

impl StreamState {
    pub fn new() -> Self {
        Self { cancelled: false }
    }
}

pub type SharedStreamState = Arc<RwLock<StreamState>>;

pub struct AIClient {
    pub client: Client,
    pub stream_state: SharedStreamState,
}

impl AIClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .connect_timeout(std::time::Duration::from_secs(15))
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(std::time::Duration::from_secs(60))
                .tcp_nodelay(true)
                .build()
                .unwrap(),
            stream_state: Arc::new(RwLock::new(StreamState::new())),
        }
    }
    
    pub async fn cancel_stream(&self) {
        let mut state = self.stream_state.write().await;
        state.cancelled = true;
    }
    
    pub async fn reset_stream_state(&self) {
        let mut state = self.stream_state.write().await;
        state.cancelled = false;
    }
    
    pub async fn is_cancelled(&self) -> bool {
        self.stream_state.read().await.cancelled
    }

    pub async fn fetch_kilo_models(&self, api_key: Option<String>, force_refresh: bool) -> Result<Vec<ModelInfo>, String> {
        if !force_refresh {
            if let Some(cached) = crate::cache::Cache::get_models() {
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
        
        let models = Self::fetch_models_from_api(&self.client, api_key).await?;
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

    pub async fn test_connection(&self, config: &AIConfig) -> Result<String, String> {
        let request = ChatRequest {
            model: config.model_name.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            stream: None,
            max_tokens: Some(10),
            temperature: None,
        };

        let mut req = self.client
            .post(&config.base_url)
            .json(&request)
            .header("Content-Type", "application/json");

        if let Some(api_key) = &config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if response.status().is_success() {
            Ok("Connection successful".to_string())
        } else {
            Err(format!("API returned: {}", response.status()))
        }
    }

    pub async fn send_chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        config: &AIConfig,
    ) -> Result<String, String> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        
        loop {
            attempts += 1;
            
            let request = ChatRequest {
                model: config.model_name.clone(),
                messages: messages.clone(),
                stream: None,
                max_tokens: None,
                temperature: None,
            };

            let mut req = self.client
                .post(&config.base_url)
                .json(&request)
                .header("Content-Type", "application/json");

            if let Some(api_key) = &config.api_key {
                req = req.header("Authorization", format!("Bearer {}", api_key));
            }

            match req.send().await {
                Ok(response) => {
                    let status = response.status();
                    
                    if !status.is_success() {
                        let body = response.text().await.unwrap_or_default();
                        
                        if status.is_client_error() {
                            return Err(format!("API error {}: {}", status, body));
                        }
                        
                        if attempts >= MAX_ATTEMPTS {
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
                        .and_then(|m| m.content.clone())
                        .unwrap_or_default();
                    
                    if content.trim().is_empty() {
                        return Err("AI returned empty response".to_string());
                    }
                    
                    return Ok(content);
                }
                Err(e) => {
                    if attempts >= MAX_ATTEMPTS {
                        return Err(format!("Request failed after {} attempts: {}", attempts, e));
                    }
                    
                    tokio::time::sleep(std::time::Duration::from_millis(500 * attempts as u64)).await;
                }
            }
        }
    }

    /// Stream chat completion with proper SSE handling
    /// Returns token usage if available
    pub async fn send_chat_completion_stream(
        &self,
        app: AppHandle,
        messages: Vec<ChatMessage>,
        config: &AIConfig,
    ) -> Result<Option<Usage>, String> {
        self.reset_stream_state().await;
        
        let request = ChatRequest {
            model: config.model_name.clone(),
            messages,
            stream: Some(true),
            max_tokens: None,
            temperature: None,
        };

        let mut req = self.client
            .post(&config.base_url)
            .json(&request)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream");

        if let Some(api_key) = &config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, body));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut done_emitted = false;
        let mut final_usage: Option<Usage> = None;
        let mut total_content = String::new();
        let mut total_reasoning = String::new();

        while let Some(chunk_result) = stream.next().await {
            // Check for cancellation
            if self.is_cancelled().await {
                log::info!("Stream cancelled by user");
                let _ = app.emit("ai-stream-chunk", "\n\n*[Generation cancelled]*");
                let _ = app.emit("ai-stream-done", ());
                return Ok(final_usage);
            }
            
            let bytes = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
            let chunk_str = String::from_utf8_lossy(&bytes);
            buffer.push_str(&chunk_str);

            // Process complete SSE events in buffer
            while let Some(event_end) = buffer.find("\n\n") {
                let event = buffer[..event_end].to_string();
                buffer = buffer[event_end + 2..].to_string();
                
                // Process the event
                for line in event.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            // Emit usage stats if available
                            if let Some(ref usage) = final_usage {
                                log::info!(
                                    "Stream complete - Tokens: prompt={}, completion={}, total={}",
                                    usage.prompt_tokens,
                                    usage.completion_tokens,
                                    usage.total_tokens
                                );
                            }
                            
                            if !done_emitted {
                                let _ = app.emit("ai-stream-done", ());
                                done_emitted = true;
                            }
                            
                            return Ok(final_usage);
                        }
                        
                        // Parse the JSON chunk
                        match serde_json::from_str::<ChatResponse>(data) {
                            Ok(chunk_data) => {
                                // Check for usage info (in final chunk before [DONE])
                                if let Some(ref usage) = chunk_data.usage {
                                    final_usage = Some(usage.clone());
                                }
                                
                                // Process choices
                                if let Some(choice) = chunk_data.choices.first() {
                                    if let Some(delta) = &choice.delta {
                                        // Handle reasoning (thinking)
                                        if let Some(reasoning) = &delta.reasoning {
                                            if !reasoning.is_empty() {
                                                total_reasoning.push_str(reasoning);
                                                let _ = app.emit("ai-stream-reasoning", reasoning.clone());
                                            }
                                        }
                                        
                                        // Handle content
                                        if let Some(content) = &delta.content {
                                            if !content.is_empty() {
                                                total_content.push_str(content);
                                                let _ = app.emit("ai-stream-chunk", content.clone());
                                            }
                                        }
                                        
                                        // Handle role assignment (first chunk)
                                        if let Some(_role) = &delta.role {
                                            log::debug!("Stream started with role: {}", _role);
                                        }
                                    }
                                    
                                    // Check for finish reason
                                    if let Some(finish_reason) = &choice.finish_reason {
                                        log::info!("Stream finished with reason: {}", finish_reason);
                                    }
                                }
                                
                                // Handle empty choices array (might contain usage)
                                if chunk_data.choices.is_empty() {
                                    if let Some(ref usage) = chunk_data.usage {
                                        final_usage = Some(usage.clone());
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse chunk: {} - Data: {}", e, 
                                          &data.chars().take(100).collect::<String>());
                            }
                        }
                    }
                }
            }
        }

        // If we reach here without [DONE], still emit done
        if !done_emitted {
            log::warn!("Stream ended without [DONE] marker");
            let _ = app.emit("ai-stream-done", ());
        }
        
        log::info!(
            "Stream finished - Content: {} chars, Reasoning: {} chars",
            total_content.len(),
            total_reasoning.len()
        );

        Ok(final_usage)
    }
}
