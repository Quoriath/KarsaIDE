use crate::ai_client::{AIClient, ModelInfo};
use crate::config_manager::{AIConfig, KarsaConfig, SessionData, load_config, save_config, update_session};
use crate::terminal::{Terminal, ShellInfo};
use crate::database::{Database, Conversation, Message};
use crate::mcp::{MCPCore, MCPRequest, MCPResponse};
use crate::workspace::{FileNode, scan_directory, quick_scan};
use crate::intelligence::{IndexEngine, CodebaseStats, ProjectMap};
use tauri::{command, AppHandle, State, Emitter};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use futures::StreamExt;
use std::path::PathBuf;

pub struct TerminalState {
    terminals: Arc<Mutex<HashMap<String, Terminal>>>,
}

pub struct IntelligenceState {
    engine: Arc<Mutex<Option<IndexEngine>>>,
}

pub struct AppState {
    db: Arc<Mutex<Database>>,
    mcp: Arc<Mutex<MCPCore>>,
}

impl IntelligenceState {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
        }
    }
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
        log::info!("Initializing AppState...");
        
        // Parallel initialization
        let db_handle = std::thread::spawn(|| {
            log::info!("Loading database...");
            Database::new().expect("Failed to initialize database")
        });
        
        let mcp_handle = std::thread::spawn(|| {
            log::info!("Loading MCP core...");
            MCPCore::new()
        });
        
        let db = db_handle.join().expect("Database init failed");
        let mcp = mcp_handle.join().expect("MCP init failed");
        
        log::info!("AppState initialized");
        
        Self {
            db: Arc::new(Mutex::new(db)),
            mcp: Arc::new(Mutex::new(mcp)),
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
pub fn check_config_exists() -> bool {
    match crate::config_manager::get_config_path() {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
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
pub async fn generate_chat_title(
    first_message: String,
    config: AIConfig,
) -> Result<String, String> {
    let client = AIClient::new();
    let messages = vec![
        crate::ai_client::ChatMessage {
            role: "system".to_string(),
            content: "Generate a short 3-5 word title for this chat. Rules: No quotes, no punctuation at end, descriptive and specific. Example: 'Fix React Hook Error' or 'Optimize Database Query'.".to_string(),
        },
        crate::ai_client::ChatMessage {
            role: "user".to_string(),
            content: first_message,
        },
    ];
    
    let title = client.send_chat_completion(messages, &config).await?;
    
    // Sanitize and validate
    let clean_title = title
        .trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '.' || c == '!' || c == '?')
        .chars()
        .take(50)  // Max 50 chars
        .collect::<String>();
    
    // Fallback if AI returns empty or invalid
    if clean_title.is_empty() || clean_title.len() < 3 {
        return Ok("New Chat".to_string());
    }
    
    Ok(clean_title)
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

#[tauri::command]
pub async fn send_agent_message_stream(
    app: AppHandle,
    state: State<'_, AppState>,
    messages: Vec<crate::ai_client::ChatMessage>,
    config: AIConfig,
) -> Result<(), String> {
    let client = AIClient::new();
    let max_iterations = 5;
    let mut iteration = 0;
    
    // Only keep last 2 messages + system prompt
    let mut conversation: Vec<crate::ai_client::ChatMessage> = messages.into_iter().rev().take(3).rev().collect();
    
    loop {
        iteration += 1;
        if iteration > max_iterations {
            let _ = app.emit("ai-stream-chunk", "\n\n⚠️ Max iterations reached".to_string());
            let _ = app.emit("ai-stream-done", ());
            break;
        }
        
        // Use STREAMING API
        let request = crate::ai_client::ChatRequest {
            model: config.model_name.clone(),
            messages: conversation.clone(),
            stream: true,
        };

        let mut req = client.client
            .post(&config.base_url)
            .json(&request);

        if let Some(api_key) = &config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req.send().await.map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        let mut stream = response.bytes_stream();
        let mut accumulated = String::new();
        
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| format!("Stream error: {}", e))?;
            let text = String::from_utf8_lossy(&bytes);

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        break;
                    }

                    if let Ok(chunk_data) = serde_json::from_str::<crate::ai_client::ChatResponse>(data) {
                        if let Some(choice) = chunk_data.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(reasoning) = &delta.reasoning {
                                    if !reasoning.is_empty() {
                                        let _ = app.emit("ai-stream-reasoning", reasoning.clone());
                                    }
                                }
                                if let Some(content) = &delta.content {
                                    if !content.is_empty() {
                                        accumulated.push_str(content);
                                        let _ = app.emit("ai-stream-chunk", content.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Check for tool calls in accumulated response
        if let Some(tool_calls) = extract_tool_calls(&accumulated) {
            let _ = app.emit("ai-stream-chunk", "\n\n".to_string());
            
            for tool_call in tool_calls {
                let tool_name = tool_call.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = tool_call.get("arguments").cloned().unwrap_or(serde_json::json!({}));
                
                // Show nice tool execution UI
                let _ = app.emit("ai-tool-call", serde_json::json!({
                    "name": tool_name,
                    "arguments": arguments
                }));
                
                let result = {
                    let mcp = state.mcp.lock().map_err(|e| format!("Lock error: {}", e))?;
                    let request = crate::mcp::MCPRequest {
                        tool: tool_name.to_string(),
                        params: arguments,
                    };
                    let response = mcp.execute(request);
                    
                    if response.success {
                        let result_str = serde_json::to_string_pretty(&response.data).unwrap_or_default();
                        let truncated = if result_str.len() > 2000 {
                            format!("{}...\n[Truncated - {} total chars]", &result_str[..2000], result_str.len())
                        } else {
                            result_str
                        };
                        format!("Tool '{}' result:\n{}", tool_name, truncated)
                    } else {
                        format!("Tool '{}' error: {}", tool_name, 
                            response.error.unwrap_or_default())
                    }
                };
                
                conversation.push(crate::ai_client::ChatMessage {
                    role: "user".to_string(),
                    content: result,
                });
            }
            
            // Keep only last 3 messages
            if conversation.len() > 3 {
                let system_msg = conversation.first().cloned();
                conversation = conversation.into_iter().rev().take(2).rev().collect();
                if let Some(sys) = system_msg {
                    if sys.role == "system" {
                        conversation.insert(0, sys);
                    }
                }
            }
            
            // Continue loop
            continue;
        }
        
        // No tool calls - done
        let _ = app.emit("ai-stream-done", ());
        break;
    }
    
    Ok(())
}

fn extract_tool_calls(text: &str) -> Option<Vec<serde_json::Value>> {
    // Look for JSON array pattern: [{...}]
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            let json_str = &text[start..=end];
            if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                if !arr.is_empty() && arr[0].get("name").is_some() {
                    return Some(arr);
                }
            }
        }
    }
    None
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
    // If mode is "all", pass None to get all conversations
    let filter_mode = mode.filter(|m| m != "all");
    db.get_conversations(filter_mode.as_deref(), limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[command]
pub fn add_message(
    app: AppHandle,
    conversation_id: i64,
    role: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let db = state.db.lock().unwrap();
    let result = db.add_message(conversation_id, &role, &content)
        .map_err(|e| e.to_string())?;
    
    // Emit event for real-time sync
    let _ = app.emit("conversation-updated", serde_json::json!({ "id": conversation_id }));
    
    Ok(result)
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

#[command]
pub fn update_conversation_title(
    app: AppHandle,
    id: i64,
    title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.update_conversation_title(id, &title)
        .map_err(|e| e.to_string())?;
    
    let _ = app.emit("conversation-updated", serde_json::json!({ "id": id }));
    Ok(())
}

#[command]
pub fn search_conversations(
    query: String,
    mode: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Conversation>, String> {
    let db = state.db.lock().unwrap();
    let filter_mode = mode.filter(|m| m != "all");
    db.search_conversations(&query, filter_mode.as_deref())
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

// MCP Commands
#[command]
pub fn mcp_execute(
    request: MCPRequest,
    state: State<'_, AppState>,
) -> MCPResponse {
    let mcp = state.mcp.lock().unwrap();
    let config = load_config();
    mcp.execute_with_policy(request, &config.security)
}

#[command]
pub fn mcp_get_tools(state: State<'_, AppState>) -> Vec<crate::mcp::ToolDefinition> {
    let mcp = state.mcp.lock().unwrap();
    mcp.get_tool_definitions()
}

#[command]
pub fn mcp_get_system_prompt(state: State<'_, AppState>) -> String {
    let mcp = state.mcp.lock().unwrap();
    mcp.generate_system_prompt()
}

#[command]
pub fn apply_smart_patch(
    path: String,
    find: String,
    replace: String,
) -> Result<serde_json::Value, String> {
    crate::mcp::smart_apply_patch(&path, &find, &replace)
        .map_err(|e| e.to_string())
}

#[command]
pub fn get_security_settings() -> crate::config_manager::SecuritySettings {
    load_config().security
}

#[command]
pub fn update_security_settings(
    settings: crate::config_manager::SecuritySettings,
) -> Result<(), String> {
    let mut config = load_config();
    config.security = settings;
    save_config(&config).map_err(|e| e.to_string())
}

// Workspace commands
#[command]
pub fn scan_workspace(path: String, depth: Option<usize>) -> Result<FileNode, String> {
    scan_directory(&path, depth.unwrap_or(2))
        .map_err(|e| e.to_string())
}

#[command]
pub fn scan_folder_shallow(path: String) -> Result<Vec<FileNode>, String> {
    quick_scan(&path)
        .map_err(|e| e.to_string())
}

#[command]
pub fn set_active_workspace(path: String) -> Result<(), String> {
    let mut config = load_config();
    
    let timestamp = chrono::Utc::now().timestamp();
    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    // Remove if exists
    config.session.recent_workspaces.retain(|w| w.path != path);
    
    // Add to front
    config.session.recent_workspaces.insert(0, crate::config_manager::RecentWorkspace {
        path: path.clone(),
        name,
        last_opened: timestamp,
    });
    
    // Keep last 10
    config.session.recent_workspaces.truncate(10);
    config.session.last_workspace = Some(path);
    
    save_config(&config).map_err(|e| e.to_string())
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

// Intelligence commands

#[command]
pub fn start_indexing(
    path: String,
    state: State<'_, IntelligenceState>,
) -> Result<(), String> {
    let engine = IndexEngine::new(PathBuf::from(path));
    engine.start_indexing();
    
    let mut eng = state.engine.lock().unwrap();
    *eng = Some(engine);
    
    Ok(())
}

#[command]
pub fn get_codebase_stats(
    state: State<'_, IntelligenceState>,
) -> Result<CodebaseStats, String> {
    let eng = state.engine.lock().unwrap();
    if let Some(engine) = eng.as_ref() {
        Ok(engine.get_stats())
    } else {
        Err("No active index".to_string())
    }
}

#[command]
pub fn get_project_map(
    state: State<'_, IntelligenceState>,
) -> Result<ProjectMap, String> {
    let eng = state.engine.lock().unwrap();
    if let Some(engine) = eng.as_ref() {
        Ok(engine.get_project_map())
    } else {
        Err("No active index".to_string())
    }
}

#[command]
pub fn query_codebase(
    query: String,
    state: State<'_, IntelligenceState>,
) -> Result<Vec<String>, String> {
    let eng = state.engine.lock().unwrap();
    if let Some(engine) = eng.as_ref() {
        Ok(engine.query_codebase(&query))
    } else {
        Err("No active index".to_string())
    }
}

#[command]
pub fn force_reindex(
    state: State<'_, IntelligenceState>,
) -> Result<(), String> {
    let eng = state.engine.lock().unwrap();
    if let Some(engine) = eng.as_ref() {
        engine.start_indexing();
        Ok(())
    } else {
        Err("No active index".to_string())
    }
}

