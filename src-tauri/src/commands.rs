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
use tokio::sync::RwLock;
use regex::Regex;

pub struct TerminalState {
    terminals: Arc<Mutex<HashMap<String, Terminal>>>,
}

pub struct IntelligenceState {
    engine: Arc<Mutex<Option<IndexEngine>>>,
}

pub struct AIState {
    client: Arc<RwLock<Option<AIClient>>>,
}

pub struct AppState {
    db: Arc<Mutex<Database>>,
    mcp: Arc<Mutex<MCPCore>>,
    ai: AIState,
}

impl AIState {
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn init_client(&self) {
        let mut client_guard = self.client.write().await;
        if client_guard.is_none() {
            *client_guard = Some(AIClient::new());
        }
    }
    
    pub async fn get_client(&self) -> Option<AIClient> {
        let client_guard = self.client.read().await;
        client_guard.as_ref().map(|c| AIClient {
            client: c.client.clone(),
            stream_state: c.stream_state.clone(),
        })
    }
    
    pub async fn cancel_stream(&self) {
        let client_guard = self.client.read().await;
        if let Some(client) = client_guard.as_ref() {
            client.cancel_stream().await;
        }
    }
    
    pub async fn reset_stream(&self) {
        let client_guard = self.client.read().await;
        if let Some(client) = client_guard.as_ref() {
            client.reset_stream_state().await;
        }
    }
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
            ai: AIState::new(),
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
    state: State<'_, AppState>,
    messages: Vec<crate::ai_client::ChatMessage>,
    config: AIConfig,
) -> Result<(), String> {
    state.ai.init_client().await;
    if let Some(client) = state.ai.get_client().await {
        client.send_chat_completion_stream(app, messages, &config).await
            .map(|_| ())
    } else {
        Err("Failed to initialize AI client".to_string())
    }
}

#[command]
pub async fn cancel_ai_stream(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.ai.cancel_stream().await;
    log::info!("AI stream cancellation requested");
    Ok(())
}

#[tauri::command]
pub async fn send_agent_message_stream(
    app: AppHandle,
    state: State<'_, AppState>,
    messages: Vec<crate::ai_client::ChatMessage>,
    config: AIConfig,
) -> Result<(), String> {
    state.ai.init_client().await;
    state.ai.reset_stream().await;
    
    let client = match state.ai.get_client().await {
        Some(c) => c,
        None => return Err("Failed to initialize AI client".to_string()),
    };
    
    let mut iteration = 0;
    const SOFT_LIMIT: i32 = 15; // Warning threshold
    const HARD_LIMIT: i32 = 25; // Emergency stop
    
    let system_msg = messages.first().filter(|m| m.role == "system").cloned();
    let user_msg = messages.last().filter(|m| m.role == "user").cloned();
    let mut conversation: Vec<crate::ai_client::ChatMessage> = messages.clone();
    
    loop {
        iteration += 1;
        log::info!("=== Agent iteration {} ===", iteration);
        
        // Soft warning
        if iteration == SOFT_LIMIT {
            log::warn!("Approaching iteration limit ({}), AI should conclude soon", SOFT_LIMIT);
        }
        
        // Hard stop (emergency)
        if iteration > HARD_LIMIT {
            log::error!("Emergency stop: exceeded {} iterations", HARD_LIMIT);
            let _ = app.emit("ai-stream-chunk", "\n\n*[Task terlalu kompleks - stopped after 25 iterations]*");
            let _ = app.emit("ai-stream-done", ());
            return Ok(());
        }
        
        if client.is_cancelled().await {
            let _ = app.emit("ai-stream-done", ());
            return Ok(());
        }
        
        // Make request
        let request = crate::ai_client::ChatRequest {
            model: config.model_name.clone(),
            messages: conversation.clone(),
            stream: Some(true),
            max_tokens: None,
            temperature: None,
        };

        let mut req = client.client
            .post(&config.base_url)
            .json(&request)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream");

        if let Some(api_key) = &config.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req.send().await.map_err(|e| format!("Request failed: {}", e))?;
        let status = response.status();
        
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(format!("API error: {} - {}", status, error_body));
        }

        // Stream response
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut full_content = String::new();
        let mut full_reasoning = String::new();
        
        while let Some(chunk_result) = stream.next().await {
            if client.is_cancelled().await {
                let _ = app.emit("ai-stream-done", ());
                return Ok(());
            }
            
            let bytes = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            while let Some(event_end) = buffer.find("\n\n") {
                let event = buffer[..event_end].to_string();
                buffer = buffer[event_end + 2..].to_string();
                
                for line in event.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            continue;
                        }
                        
                        if let Ok(chunk_data) = serde_json::from_str::<crate::ai_client::ChatResponse>(data) {
                            if let Some(choice) = chunk_data.choices.first() {
                                if let Some(delta) = &choice.delta {
                                    if let Some(reasoning) = &delta.reasoning {
                                        if !reasoning.is_empty() {
                                            full_reasoning.push_str(reasoning);
                                            let _ = app.emit("ai-stream-reasoning", reasoning.clone());
                                        }
                                    }
                                    
                                    if let Some(content) = &delta.content {
                                        if !content.is_empty() {
                                            full_content.push_str(content);
                                            // Don't stream yet - wait to check for tool calls
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        log::info!("Stream complete - Content: {} chars", full_content.len());
        
        // Emit complete reasoning first
        if !full_reasoning.is_empty() {
            let _ = app.emit("ai-reasoning-complete", full_reasoning.clone());
        }
        
        // Check if content contains tool calls
        let (text_before, tool_calls, text_after) = extract_tool_calls_with_context(&full_content);
        
        if !tool_calls.is_empty() {
            log::info!("Found {} tool calls", tool_calls.len());
            
            // Emit text BEFORE tool calls
            if !text_before.is_empty() {
                let _ = app.emit("ai-stream-chunk", text_before.clone());
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            // Process ONLY FIRST tool (enforce sequential)
            let (tool_name, arguments) = &tool_calls[0];
            
            // Emit tool call event
            let _ = app.emit("ai-tool-call", serde_json::json!({
                "name": tool_name,
                "arguments": arguments
            }));
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
            // Execute tool with timeout
            let tool_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(30),
                async {
                    execute_mcp_tool(&state, tool_name, arguments.clone())
                }
            ).await;
            
            let (result_text, is_error) = match tool_result {
                Ok(Ok(data)) => {
                    let result_str = serde_json::to_string_pretty(&data).unwrap_or_else(|_| "null".to_string());
                    let truncated = if result_str.len() > 2000 {
                        format!("{}...\n\n[Result truncated: {} total chars]", &result_str[..2000], result_str.len())
                    } else if result_str == "null" {
                        "Success (no output).".to_string()
                    } else {
                        result_str
                    };
                    (truncated, false)
                }
                Ok(Err(e)) => (format!("Tool error: {}", e), true),
                Err(_) => (format!("Timeout: Tool took longer than 30 seconds"), true),
            };
            
            // Emit result to frontend
            let _ = app.emit("ai-tool-result", serde_json::json!({
                "name": tool_name,
                "result": result_text.clone(),
                "error": if is_error { Some(result_text.clone()) } else { None },
                "isError": is_error
            }));
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Build conversation for next iteration
            conversation.clear();
            
            if let Some(ref sys) = system_msg {
                conversation.push(sys.clone());
            }
            
            if let Some(ref um) = user_msg {
                conversation.push(um.clone());
            }
            
            // Add tool execution context
            conversation.push(crate::ai_client::ChatMessage {
                role: "assistant".to_string(),
                content: if !text_before.is_empty() {
                    format!("{}\n\n[Executed: {}]", text_before, tool_name)
                } else {
                    format!("[Executed: {}]", tool_name)
                },
            });
            
            conversation.push(crate::ai_client::ChatMessage {
                role: "user".to_string(),
                content: if is_error {
                    format!("Tool '{}' failed:\n{}\n\nPlease try a different approach or explain the issue.", tool_name, result_text)
                } else {
                    format!("Tool '{}' result:\n{}\n\nNow continue your response based on this result.", tool_name, result_text)
                },
            });
            
            // Continue loop - AI will respond to tool result
            continue;
        }
        
        // No tool calls - stream the content and finish
        if !full_content.is_empty() {
            let _ = app.emit("ai-stream-chunk", full_content.clone());
        }
        
        let _ = app.emit("ai-stream-done", ());
        break;
    }
    
    Ok(())
}

fn extract_tool_calls_with_context(text: &str) -> (String, Vec<(String, serde_json::Value)>, String) {
    let pattern = Regex::new(r#"\[\s*\{\s*"name"\s*:\s*"([^"]+)"\s*,\s*"arguments"\s*:\s*(\{[^{}]*\})\s*\}\s*\]"#).unwrap();
    
    let mut tool_calls = Vec::new();
    let mut text_before = String::new();
    let mut text_after = String::new();
    
    if let Some(first_match) = pattern.find(text) {
        // Text before first tool call
        text_before = text[..first_match.start()].trim().to_string();
        
        // Extract tool call
        if let Some(cap) = pattern.captures(first_match.as_str()) {
            if let (Some(name_match), Some(args_match)) = (cap.get(1), cap.get(2)) {
                let tool_name = name_match.as_str().to_string();
                let args_str = args_match.as_str();
                
                if let Ok(arguments) = serde_json::from_str::<serde_json::Value>(args_str) {
                    if tool_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        tool_calls.push((tool_name, arguments));
                    }
                }
            }
        }
        
        // Text after tool call (if any)
        text_after = text[first_match.end()..].trim().to_string();
    } else {
        // No tool calls found
        text_before = text.trim().to_string();
    }
    
    (text_before, tool_calls, text_after)
}

fn extract_tool_calls(text: &str) -> (String, Vec<(String, serde_json::Value)>) {
    let mut clean_text = text.to_string();
    let mut tool_calls = Vec::new();
    
    // Pattern to find tool calls
    let pattern = Regex::new(r#"\[\s*\{\s*"name"\s*:\s*"([^"]+)"\s*,\s*"arguments"\s*:\s*(\{[^{}]*\})\s*\}\s*\]"#).unwrap();
    
    // Find all tool calls
    for cap in pattern.captures_iter(text) {
        if let (Some(name_match), Some(args_match)) = (cap.get(1), cap.get(2)) {
            let tool_name = name_match.as_str().to_string();
            let args_str = args_match.as_str();
            
            if let Ok(arguments) = serde_json::from_str::<serde_json::Value>(args_str) {
                if tool_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    tool_calls.push((tool_name, arguments));
                }
            }
        }
    }
    
    // Remove all tool call JSONs from text
    clean_text = pattern.replace_all(&clean_text, "").to_string();
    
    // Clean up extra whitespace
    clean_text = clean_text.split_whitespace().collect::<Vec<_>>().join(" ");
    
    (clean_text, tool_calls)
}

fn execute_mcp_tool(
    state: &State<'_, AppState>,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, anyhow::Error> {
    let mcp = state.mcp.lock()
        .map_err(|e| anyhow::anyhow!("Failed to acquire MCP lock: {}", e))?;
    
    let request = crate::mcp::MCPRequest {
        tool: tool_name.to_string(),
        params: arguments,
    };
    
    let config = load_config();
    let response = mcp.execute_with_policy(request, &config.security);
    
    if response.success {
        Ok(response.data.unwrap_or(serde_json::json!(null)))
    } else {
        Err(anyhow::anyhow!("{}", response.error.unwrap_or_else(|| "Unknown error".to_string())))
    }
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
pub fn mcp_get_system_prompt(
    state: State<'_, AppState>,
    mode: Option<String>,
    cwd: Option<String>,
) -> String {
    let mcp = state.mcp.lock().unwrap();
    let mode = mode.unwrap_or_else(|| "code".to_string());
    let cwd = cwd.unwrap_or_else(|| ".".to_string());
    let prompt = mcp.generate_system_prompt(&mode, &cwd);
    log::info!("Generated system prompt: {} chars", prompt.len());
    prompt
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
    let terminal = Terminal::spawn(app, id.clone(), shell)
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

#[command]
pub fn close_terminal(
    id: String,
    state: State<'_, TerminalState>,
) -> Result<(), String> {
    state.terminals.lock().unwrap().remove(&id);
    Ok(())
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

