use crate::ai_client::{AIClient, ModelInfo};
use crate::config_manager::{AIConfig, KarsaConfig, SessionData, load_config, save_config, update_session};
use crate::terminal::{Terminal, ShellInfo};
use crate::database::{Database, Conversation, Message};
use tauri::{command, AppHandle, State, Emitter};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct TerminalState {
    terminals: Arc<Mutex<HashMap<String, Terminal>>>,
}

pub struct AppState {
    db: Arc<Mutex<Database>>,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            terminals: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        let db = Database::new().expect("Failed to initialize database");
        Self {
            db: Arc::new(Mutex::new(db)),
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
pub async fn fetch_kilo_models(api_key: Option<String>, force_refresh: Option<bool>) -> Result<Vec<ModelInfo>, String> {
    let client = AIClient::new();
    client.fetch_kilo_models(api_key, force_refresh.unwrap_or(false)).await
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

// Database commands
#[command]
pub fn create_conversation(
    mode: String,
    title: String,
    context_path: Option<String>,
    model: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let db = state.db.lock().unwrap();
    db.create_conversation(&mode, &title, context_path.as_deref(), &model)
        .map_err(|e| e.to_string())
}

#[command]
pub fn get_conversations(
    mode: Option<String>,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<Conversation>, String> {
    let db = state.db.lock().unwrap();
    db.get_conversations(mode.as_deref(), limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[command]
pub fn add_message(
    conversation_id: i64,
    role: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let db = state.db.lock().unwrap();
    db.add_message(conversation_id, &role, &content)
        .map_err(|e| e.to_string())
}

#[command]
pub fn get_messages(
    conversation_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<Message>, String> {
    let db = state.db.lock().unwrap();
    db.get_messages(conversation_id)
        .map_err(|e| e.to_string())
}

#[command]
pub fn delete_conversation(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.delete_conversation(id)
        .map_err(|e| e.to_string())
}

// Session commands
#[command]
pub fn get_session() -> SessionData {
    load_config().session
}

#[command]
pub fn save_session(session: SessionData) -> Result<(), String> {
    update_session(session).map_err(|e| e.to_string())
}

// UI state events
#[command]
pub fn toggle_terminal(app: AppHandle, visible: bool) -> Result<(), String> {
    app.emit("terminal-state-changed", visible)
        .map_err(|e| e.to_string())
}

#[command]
pub fn toggle_chat(app: AppHandle, visible: bool) -> Result<(), String> {
    app.emit("chat-state-changed", visible)
        .map_err(|e| e.to_string())
}

// Terminal commands
#[command]
pub fn get_shell_info() -> ShellInfo {
    Terminal::get_shell_info()
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

