use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub tool: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PatchRequest {
    pub path: String,
    pub find: String,
    pub replace: String,
}

pub struct MCPCore {
    tools: HashMap<String, Box<dyn MCPTool + Send + Sync>>,
    initialized: bool,
}

pub trait MCPTool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ToolParameter>;
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value>;
    fn category(&self) -> ToolCategory {
        ToolCategory::Safe
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ToolCategory {
    Safe,           // Auto-execute (read, code edits)
    Destructive,    // Check config (delete, shell)
}

impl MCPCore {
    pub fn execute_with_policy(
        &self,
        request: MCPRequest,
        security: &crate::config_manager::SecuritySettings,
    ) -> MCPResponse {
        match self.tools.get(&request.tool) {
            Some(tool) => {
                let category = tool.category();
                
                // Category A: Safe operations - always execute
                if category == ToolCategory::Safe {
                    return match tool.execute(request.params) {
                        Ok(data) => MCPResponse {
                            success: true,
                            data: Some(data),
                            error: None,
                        },
                        Err(e) => MCPResponse {
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                        },
                    };
                }
                
                // Category B: Destructive - check config
                let auto_execute = match request.tool.as_str() {
                    "file_delete" => security.auto_delete_files,
                    "file_move" => security.auto_move_files,
                    "execute_command" => security.auto_execute_shell,
                    _ => false,
                };
                
                if auto_execute {
                    match tool.execute(request.params) {
                        Ok(data) => MCPResponse {
                            success: true,
                            data: Some(data),
                            error: None,
                        },
                        Err(e) => MCPResponse {
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                        },
                    }
                } else {
                    MCPResponse {
                        success: false,
                        data: None,
                        error: Some("REQUIRES_APPROVAL".to_string()),
                    }
                }
            }
            None => MCPResponse {
                success: false,
                data: None,
                error: Some(format!("Tool '{}' not found", request.tool)),
            },
        }
    }
}

impl MCPCore {
    pub fn new() -> Self {
        log::info!("Initializing MCP Core...");
        let mut core = Self {
            tools: HashMap::new(),
            initialized: false,
        };
        
        // Register built-in tools (fast, no I/O)
        core.register_tool(Box::new(FileReadTool));
        core.register_tool(Box::new(FileReadRangeTool));
        core.register_tool(Box::new(ListSymbolsTool));
        core.register_tool(Box::new(FileWriteTool));
        core.register_tool(Box::new(FileMoveTool));
        core.register_tool(Box::new(FileCopyTool));
        core.register_tool(Box::new(FileDeleteTool));
        core.register_tool(Box::new(SearchTool));
        core.register_tool(Box::new(ListFilesTool));
        core.register_tool(Box::new(GetFileInfoTool));
        core.register_tool(Box::new(SmartPatchTool));
        
        core.initialized = true;
        log::info!("MCP Core initialized with {} tools", core.tools.len());
        
        core
    }
    
    pub fn register_tool(&mut self, tool: Box<dyn MCPTool + Send + Sync>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    #[allow(dead_code)]
    pub fn execute(&self, request: MCPRequest) -> MCPResponse {
        match self.tools.get(&request.tool) {
            Some(tool) => match tool.execute(request.params) {
                Ok(data) => MCPResponse {
                    success: true,
                    data: Some(data),
                    error: None,
                },
                Err(e) => MCPResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                },
            },
            None => MCPResponse {
                success: false,
                data: None,
                error: Some(format!("Tool '{}' not found", request.tool)),
            },
        }
    }
    
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| ToolDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameters(),
            })
            .collect()
    }
    
    pub fn generate_system_prompt(&self) -> String {
        let tools_json = serde_json::to_string_pretty(&self.get_tool_definitions()).unwrap();
        
        format!(
            r#"You are Karsa, a highly skilled software engineer with extensive knowledge in programming languages, frameworks, design patterns, and best practices.

====

TOOL USE

You have access to MCP tools for codebase analysis. You must use exactly one tool call per response. Do not call zero tools or more than one tool in the same response.

# Tool Use Guidelines

1. Assess what information you already have and what you need to proceed.
2. Choose the most appropriate tool based on the task. Think about each available tool and use the one that best fits the current step.
3. If multiple actions are needed, use one tool at a time per message, with each tool use informed by the previous result.
4. After each tool use, wait for the result before proceeding. This result will provide necessary information to continue.

# Tool Call Format (EXACT)

[{{"name": "tool_name", "arguments": {{"param": "value"}}}}]

❌ WRONG: {{"tool": "name"}} or {{"name": "tool_name"}}
✅ RIGHT: [{{"name": "tool_name", "arguments": {{}}}}]

====

CAPABILITIES

- You have access to tools that let you list files, view source code, regex search, and read files with smart range support.
- When the user gives you a task, use get_project_map to get an overview of the project structure.
- You can use search with regex patterns for semantic search (e.g., "class.*Component", "fn\\s+\\w+").
- file_read supports smart reading: defaults to first 500 lines, or specify start_line/end_line for ranges.
- Use get_file_info to check file size before reading large files.

====

RULES

- STRICTLY FORBIDDEN from starting messages with "Great", "Certainly", "Okay", "Sure". Be direct and technical.
- Do NOT be conversational. Say "I've updated the CSS" NOT "Great, I've updated the CSS".
- Your goal is to accomplish the task, NOT engage in back and forth conversation.
- NEVER end responses with questions or offers for further assistance.
- Work through tasks sequentially, one tool at a time.
- Wait for confirmation after each tool use before proceeding.
- Be clear and technical in your messages.

====

CRITICAL CONSTRAINTS (HARD ENFORCED)

1. TOKEN LIMITS:
   - Max 5 tool calls per conversation
   - Max 1000 lines per file_read
   - Tool results truncated to 2000 chars

2. MANDATORY WORKFLOW:
   Step 1: ALWAYS call get_project_map() first on new tasks
   Step 2: Use search() to find relevant files
   Step 3: Use get_file_info() to check file size
   Step 4: Use file_read() with appropriate range
   
   ❌ NEVER skip get_project_map on first interaction
   ❌ NEVER read files without checking size first
   ❌ NEVER request >1000 lines in one call

3. SEARCH RULES:
   - Use specific patterns: "fn login", "class User"
   - Use regex for semantic: "class.*Component", "fn\\s+\\w+"
   - Add file_type filter: {{"file_type": "rs"}}
   - Max 50 results per search

4. FILE READING:
   - Default: first 500 lines (no params needed)
   - Range: {{"start_line": 100, "end_line": 200}}
   - Large files: list_symbols first, then file_read with range
   - Hard limit: 1000 lines max

====

AVAILABLE TOOLS

{}

====

EXAMPLES

User: "Show me the project"
You: [{{"name": "get_project_map", "arguments": {{}}}}]
System: (returns structure)
You: "Project structure:\n- src/ (15 files)\n- tests/ (8 files)\n..."

User: "Find all React components"
You: [{{"name": "search", "arguments": {{"pattern": "class.*Component", "file_type": "js"}}}}]
System: (returns matches)
You: "Found 12 components:\n1. UserComponent (src/User.js:15)\n..."

User: "Read main.rs lines 100-200"
You: [{{"name": "file_read", "arguments": {{"path": "./src/main.rs", "start_line": 100, "end_line": 200}}}}]
System: (returns lines)
You: "Lines 100-200 of main.rs:\n```rust\n...\n```"

====

OBJECTIVE

You accomplish tasks iteratively:

1. Analyze the task and set clear, achievable goals in logical order.
2. Work through goals sequentially, using tools one at a time.
3. Before calling a tool, analyze which tool is most relevant and if you have all required parameters.
4. If parameters are missing, ask for them. Do NOT invoke tools with placeholder values.
5. Once task is complete, provide the result directly without asking for feedback.

====

RESPONSE FORMAT

1. If you need data → return JSON array ONLY (one tool call)
2. After receiving results → provide natural language answer
3. Use code blocks with syntax highlighting: ```language
4. Cite file paths and line numbers
5. Be concise and technical
6. NO conversational fluff
7. NO questions at the end

====

START: When given a new task, call get_project_map() first to understand the codebase structure."#,
            tools_json
        )
    }
}
// Built-in Tools
struct FileReadTool;
impl MCPTool for FileReadTool {
    fn name(&self) -> &str { "file_read" }
    fn description(&self) -> &str { 
        "Read file contents. Default: first 500 lines. Optional: specify start_line and end_line for range reading." 
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "File path (required)".to_string(),
                required: true,
            },
            ToolParameter {
                name: "start_line".to_string(),
                param_type: "number".to_string(),
                description: "Start line (1-indexed, optional, default: 1)".to_string(),
                required: false,
            },
            ToolParameter {
                name: "end_line".to_string(),
                param_type: "number".to_string(),
                description: "End line (1-indexed, optional, default: 500)".to_string(),
                required: false,
            },
        ]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let path = params["path"].as_str().ok_or(anyhow::anyhow!("Missing path"))?;
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        // Get range parameters (default: first 500 lines)
        let start = params["start_line"].as_u64().unwrap_or(1).max(1) as usize;
        let end = params["end_line"].as_u64().unwrap_or(500).min(total_lines as u64) as usize;
        
        // Validate range
        if start > total_lines {
            return Err(anyhow::anyhow!("start_line {} exceeds file length {}", start, total_lines));
        }
        
        let actual_end = end.min(total_lines);
        let range_size = actual_end - start + 1;
        
        // Hard limit: max 1000 lines per read
        if range_size > 1000 {
            return Err(anyhow::anyhow!(
                "Range too large ({} lines). Max 1000 lines per read. Use smaller ranges.",
                range_size
            ));
        }
        
        let selected_lines = &lines[(start - 1)..actual_end];
        let result_content = selected_lines.join("\n");
        
        Ok(serde_json::json!({ 
            "content": result_content,
            "start_line": start,
            "end_line": actual_end,
            "lines_returned": selected_lines.len(),
            "total_lines": total_lines,
            "truncated": actual_end < total_lines
        }))
    }
}

struct FileReadRangeTool;
impl MCPTool for FileReadRangeTool {
    fn name(&self) -> &str { "file_read_range" }
    fn description(&self) -> &str { "Read specific line range from file" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "File path".to_string(),
                required: true,
            },
            ToolParameter {
                name: "start_line".to_string(),
                param_type: "number".to_string(),
                description: "Start line (1-indexed)".to_string(),
                required: true,
            },
            ToolParameter {
                name: "end_line".to_string(),
                param_type: "number".to_string(),
                description: "End line (inclusive)".to_string(),
                required: true,
            },
        ]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let path = params["path"].as_str().ok_or(anyhow::anyhow!("Missing path"))?;
        let start = params["start_line"].as_u64().ok_or(anyhow::anyhow!("Missing start_line"))? as usize;
        let end = params["end_line"].as_u64().ok_or(anyhow::anyhow!("Missing end_line"))? as usize;
        
        if end - start > 500 {
            return Err(anyhow::anyhow!("Range too large. Max 500 lines per read."));
        }
        
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        if start < 1 || start > lines.len() || end > lines.len() {
            return Err(anyhow::anyhow!("Invalid line range. File has {} lines", lines.len()));
        }
        
        let range_content = lines[(start-1)..end].join("\n");
        
        Ok(serde_json::json!({ 
            "content": range_content,
            "start": start,
            "end": end,
            "lines_read": end - start + 1,
            "total_lines": lines.len()
        }))
    }
}

struct ListSymbolsTool;
impl MCPTool for ListSymbolsTool {
    fn name(&self) -> &str { "list_symbols" }
    fn description(&self) -> &str { "List functions/classes in file without reading content" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "File path".to_string(),
            required: true,
        }]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let path = params["path"].as_str().ok_or(anyhow::anyhow!("Missing path"))?;
        let content = std::fs::read_to_string(path)?;
        
        // Simple symbol extraction (functions, classes)
        let mut symbols = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") || 
               trimmed.starts_with("pub fn ") ||
               trimmed.starts_with("async fn ") ||
               trimmed.starts_with("class ") ||
               trimmed.starts_with("function ") ||
               trimmed.starts_with("def ") {
                symbols.push(serde_json::json!({
                    "line": i + 1,
                    "symbol": trimmed.split('(').next().unwrap_or(trimmed)
                }));
            }
        }
        
        Ok(serde_json::json!({ 
            "symbols": symbols,
            "file": path
        }))
    }
}

struct FileWriteTool;
impl MCPTool for FileWriteTool {
    fn name(&self) -> &str { "file_write" }
    fn description(&self) -> &str { "Write file contents" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "File path".to_string(),
                required: true,
            },
            ToolParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "File content".to_string(),
                required: true,
            },
        ]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let path = params["path"].as_str().ok_or(anyhow::anyhow!("Missing path"))?;
        let content = params["content"].as_str().ok_or(anyhow::anyhow!("Missing content"))?;
        std::fs::write(path, content)?;
        Ok(serde_json::json!({ "success": true }))
    }
}

struct SearchTool;
impl MCPTool for SearchTool {
    fn name(&self) -> &str { "search" }
    fn description(&self) -> &str { 
        "Semantic search in codebase. Supports regex patterns (e.g., 'class.*Component', 'fn\\s+login'). Returns file paths and line numbers with context." 
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Search pattern (supports regex, e.g., 'class.*', 'fn\\s+\\w+')".to_string(),
                required: true,
            },
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Search path (default: current directory)".to_string(),
                required: false,
            },
            ToolParameter {
                name: "file_type".to_string(),
                param_type: "string".to_string(),
                description: "Filter by file extension (e.g., 'rs', 'js', 'py')".to_string(),
                required: false,
            },
        ]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let pattern = params["pattern"].as_str().ok_or(anyhow::anyhow!("Missing pattern"))?;
        let path = params["path"].as_str().unwrap_or(".");
        let file_type = params["file_type"].as_str();
        
        let mut cmd = std::process::Command::new("rg");
        cmd.arg(pattern)
            .arg(path)
            .arg("--json")
            .arg("--max-count=50")
            .arg("--context=2")
            .arg("--smart-case");
        
        if let Some(ft) = file_type {
            cmd.arg("--type").arg(ft);
        }
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            return Ok(serde_json::json!({ 
                "matches": [],
                "message": "No matches found"
            }));
        }
        
        let results = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = results.lines().collect();
        
        // Parse ripgrep JSON output
        let mut matches = Vec::new();
        for line in lines {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json["type"] == "match" {
                    matches.push(json);
                }
            }
        }
        
        Ok(serde_json::json!({ 
            "matches": matches,
            "total": matches.len(),
            "pattern": pattern
        }))
    }
}

struct FileMoveTool;
impl MCPTool for FileMoveTool {
    fn name(&self) -> &str { "file_move" }
    fn description(&self) -> &str { "Move/rename file" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "from".to_string(),
                param_type: "string".to_string(),
                description: "Source path".to_string(),
                required: true,
            },
            ToolParameter {
                name: "to".to_string(),
                param_type: "string".to_string(),
                description: "Destination path".to_string(),
                required: true,
            },
        ]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let from = params["from"].as_str().ok_or(anyhow::anyhow!("Missing from"))?;
        let to = params["to"].as_str().ok_or(anyhow::anyhow!("Missing to"))?;
        std::fs::rename(from, to)?;
        Ok(serde_json::json!({ "success": true, "from": from, "to": to }))
    }
}

struct FileCopyTool;
impl MCPTool for FileCopyTool {
    fn name(&self) -> &str { "file_copy" }
    fn description(&self) -> &str { "Copy file" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "from".to_string(),
                param_type: "string".to_string(),
                description: "Source path".to_string(),
                required: true,
            },
            ToolParameter {
                name: "to".to_string(),
                param_type: "string".to_string(),
                description: "Destination path".to_string(),
                required: true,
            },
        ]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let from = params["from"].as_str().ok_or(anyhow::anyhow!("Missing from"))?;
        let to = params["to"].as_str().ok_or(anyhow::anyhow!("Missing to"))?;
        std::fs::copy(from, to)?;
        Ok(serde_json::json!({ "success": true, "from": from, "to": to }))
    }
}

struct FileDeleteTool;
impl MCPTool for FileDeleteTool {
    fn name(&self) -> &str { "file_delete" }
    fn description(&self) -> &str { "Delete file" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "File path".to_string(),
            required: true,
        }]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let path = params["path"].as_str().ok_or(anyhow::anyhow!("Missing path"))?;
        std::fs::remove_file(path)?;
        Ok(serde_json::json!({ "success": true, "deleted": path }))
    }
}

struct ListFilesTool;
impl MCPTool for ListFilesTool {
    fn name(&self) -> &str { "list_files" }
    fn description(&self) -> &str { "List files in directory" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Directory path".to_string(),
                required: true,
            },
            ToolParameter {
                name: "recursive".to_string(),
                param_type: "boolean".to_string(),
                description: "Recursive listing".to_string(),
                required: false,
            },
        ]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let path = params["path"].as_str().ok_or(anyhow::anyhow!("Missing path"))?;
        let recursive = params["recursive"].as_bool().unwrap_or(false);
        
        let mut files = Vec::new();
        
        if recursive {
            for entry in walkdir::WalkDir::new(path).max_depth(5) {
                if let Ok(entry) = entry {
                    if entry.file_type().is_file() {
                        files.push(entry.path().display().to_string());
                    }
                }
            }
        } else {
            for entry in std::fs::read_dir(path)? {
                if let Ok(entry) = entry {
                    files.push(entry.path().display().to_string());
                }
            }
        }
        
        Ok(serde_json::json!({ "files": files, "count": files.len() }))
    }
}

struct GetFileInfoTool;
impl MCPTool for GetFileInfoTool {
    fn name(&self) -> &str { "get_file_info" }
    fn description(&self) -> &str { "Get file metadata (size, lines, type)" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "File path".to_string(),
            required: true,
        }]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let path = params["path"].as_str().ok_or(anyhow::anyhow!("Missing path"))?;
        let metadata = std::fs::metadata(path)?;
        
        let lines = if metadata.is_file() {
            std::fs::read_to_string(path)?.lines().count()
        } else {
            0
        };
        
        Ok(serde_json::json!({
            "path": path,
            "size_bytes": metadata.len(),
            "lines": lines,
            "is_file": metadata.is_file(),
            "is_dir": metadata.is_dir(),
            "readonly": metadata.permissions().readonly()
        }))
    }
}

// Smart Patch System
pub fn smart_apply_patch(path: &str, find: &str, replace: &str) -> Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)?;
    
    // Normalize whitespace for matching
    let find_normalized = find.trim();
    let find_lines: Vec<&str> = find_normalized.lines().collect();
    
    if find_lines.is_empty() {
        return Err(anyhow::anyhow!("Find pattern is empty"));
    }
    
    // Find exact match first
    if let Some(pos) = content.find(find_normalized) {
        let new_content = content.replacen(find_normalized, replace.trim(), 1);
        std::fs::write(path, &new_content)?;
        
        return Ok(serde_json::json!({
            "success": true,
            "method": "exact_match",
            "position": pos
        }));
    }
    
    // Fuzzy context match
    let content_lines: Vec<&str> = content.lines().collect();
    let mut best_match: Option<(usize, usize, f32)> = None;
    
    for i in 0..content_lines.len() {
        if i + find_lines.len() > content_lines.len() {
            break;
        }
        
        let mut matches = 0;
        let mut total = 0;
        
        for (j, find_line) in find_lines.iter().enumerate() {
            let content_line = content_lines[i + j].trim();
            let find_line_trim = find_line.trim();
            
            if content_line == find_line_trim {
                matches += 1;
            }
            total += 1;
        }
        
        let similarity = matches as f32 / total as f32;
        
        if similarity > 0.8 {
            best_match = Some((i, i + find_lines.len(), similarity));
            break;
        }
    }
    
    if let Some((start, end, similarity)) = best_match {
        // Get indentation from first line
        let indent = content_lines[start]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>();
        
        // Apply indentation to replacement
        let replace_lines: Vec<String> = replace
            .trim()
            .lines()
            .map(|line| {
                if line.trim().is_empty() {
                    String::new()
                } else {
                    format!("{}{}", indent, line.trim())
                }
            })
            .collect();
        
        // Reconstruct file
        let mut new_lines = content_lines[..start].to_vec();
        new_lines.extend(replace_lines.iter().map(|s| s.as_str()));
        new_lines.extend(&content_lines[end..]);
        
        let new_content = new_lines.join("\n");
        std::fs::write(path, &new_content)?;
        
        return Ok(serde_json::json!({
            "success": true,
            "method": "fuzzy_match",
            "similarity": similarity,
            "line_start": start + 1,
            "line_end": end
        }));
    }
    
    Err(anyhow::anyhow!(
        "Could not find unique match. Pattern not found or ambiguous."
    ))
}

struct SmartPatchTool;
impl MCPTool for SmartPatchTool {
    fn name(&self) -> &str { "smart_patch" }
    fn description(&self) -> &str { "Apply code patch with fuzzy context matching" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "File path".to_string(),
                required: true,
            },
            ToolParameter {
                name: "find".to_string(),
                param_type: "string".to_string(),
                description: "Code to find".to_string(),
                required: true,
            },
            ToolParameter {
                name: "replace".to_string(),
                param_type: "string".to_string(),
                description: "Replacement code".to_string(),
                required: true,
            },
        ]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let path = params["path"].as_str().ok_or(anyhow::anyhow!("Missing path"))?;
        let find = params["find"].as_str().ok_or(anyhow::anyhow!("Missing find"))?;
        let replace = params["replace"].as_str().ok_or(anyhow::anyhow!("Missing replace"))?;
        
        smart_apply_patch(path, find, replace)
    }
}

