use crate::ai_client::{AIClient, ModelInfo};
use crate::config_manager::{AIConfig, KarsaConfig, SessionData, load_config, save_config, update_session};
use crate::terminal::Terminal;
use tauri::{command, AppHandle, State};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct TerminalState {
    terminals: Arc<Mutex<HashMap<String, Terminal>>>,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            terminals: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[command]
pub fn get_ai_config() -> KarsaConfig {
    load_config()
}

#[command]
pub fn save_ai_config(config: KarsaConfig) -> Result<(), String> {
    save_config(&config).map_err(|e| e.to_string())
}

#[command]
pub async fn fetch_kilo_models(api_key: Option<String>) -> Result<Vec<ModelInfo>, String> {
    let client = AIClient::new();
    client.fetch_kilo_models(api_key).await
}

#[command]
pub async fn test_ai_connection(config: AIConfig) -> Result<String, String> {
    let client = AIClient::new();
    client.test_connection(&config).await
}

#[command]
pub async fn send_chat_completion(
    messages: Vec<crate::ai_client::ChatMessage>,
    config: AIConfig,
) -> Result<String, String> {
    let client = AIClient::new();
    client.send_chat_completion(messages, &config).await
}

#[command]
pub async fn send_chat_completion_stream(
    app: AppHandle,
    messages: Vec<crate::ai_client::ChatMessage>,
    config: AIConfig,
) -> Result<(), String> {
    let client = AIClient::new();
    client.send_chat_completion_stream(app, messages, &config).await
}

#[command]
pub fn get_session() -> SessionData {
    load_config().session
}

#[command]
pub fn save_session(session: SessionData) -> Result<(), String> {
    update_session(session).map_err(|e| e.to_string())
}

#[command]
pub fn spawn_terminal(
    app: AppHandle,
    id: String,
    shell: Option<String>,
    state: State<'_, TerminalState>,
) -> Result<(), String> {
    let terminal = Terminal::spawn(app, shell)
        .map_err(|e| format!("Failed to spawn terminal: {}", e))?;
    
    state.terminals.lock().unwrap().insert(id, terminal);
    Ok(())
}

#[command]
pub fn write_to_terminal(
    id: String,
    data: Vec<u8>,
    state: State<'_, TerminalState>,
) -> Result<(), String> {
    let terminals = state.terminals.lock().unwrap();
    if let Some(terminal) = terminals.get(&id) {
        terminal.write(&data)
            .map_err(|e| format!("Failed to write to terminal: {}", e))
    } else {
        Err("Terminal not found".to_string())
    }
}

