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
            r#"AVAILABLE MCP TOOLS:
{}

TOOL USAGE PROTOCOL:
1. SEARCH FIRST: Use 'search' to find relevant files
2. MAP STRUCTURE: Use 'list_symbols' to see functions/classes
3. READ PRECISELY: Use 'file_read_range' for specific lines

CONTEXT LIMITS:
- file_read: MAX 1000 lines
- file_read_range: MAX 500 lines per call
- ALWAYS check file size with get_file_info first
- Use list_symbols before reading large files

RESPONSE FORMAT:
- Answer in natural language
- When using tools, explain what you're doing
- Cite file paths and line numbers
- Be direct and concise

WORKFLOW EXAMPLE:
User: "Find authentication code"
You: "Let me search for authentication files..."
[Use search tool]
You: "Found src/auth.rs. Let me check its structure..."
[Use list_symbols]
You: "I can see the login function at lines 45-80. Here's what it does..."
[Use file_read_range]

NEVER:
- Return raw JSON as response
- Read entire large files without checking size
- Guess about code without using tools"#,
            tools_json
        )
    }
}
// Built-in Tools
struct FileReadTool;
impl MCPTool for FileReadTool {
    fn name(&self) -> &str { "file_read" }
    fn description(&self) -> &str { "Read file contents (max 1000 lines, use file_read_range for larger files)" }
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
        let lines: Vec<&str> = content.lines().collect();
        
        // Context Economy: Reject very large files
        if lines.len() > 1000 {
            return Err(anyhow::anyhow!(
                "File too large ({} lines). Use file_read_range(path, start, end) instead. Max 1000 lines per read.",
                lines.len()
            ));
        }
        
        Ok(serde_json::json!({ 
            "content": content,
            "lines": lines.len(),
            "size_bytes": content.len()
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
    fn description(&self) -> &str { "Search codebase with ripgrep" }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Search pattern".to_string(),
                required: true,
            },
            ToolParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Search path".to_string(),
                required: false,
            },
        ]
    }
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let pattern = params["pattern"].as_str().ok_or(anyhow::anyhow!("Missing pattern"))?;
        let path = params["path"].as_str().unwrap_or(".");
        
        let output = std::process::Command::new("rg")
            .arg(pattern)
            .arg(path)
            .arg("--json")
            .arg("--max-count=50")
            .output()?;
        
        let results = String::from_utf8_lossy(&output.stdout);
        Ok(serde_json::json!({ "results": results }))
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

