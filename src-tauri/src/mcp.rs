use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

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

pub struct MCPCore {
    tools: HashMap<String, Box<dyn MCPTool + Send + Sync>>,
    initialized: bool,
}

pub trait MCPTool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ToolParameter>;
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value>;
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
        core.register_tool(Box::new(SearchTool));
        
        core.initialized = true;
        log::info!("MCP Core initialized with {} tools", core.tools.len());
        
        core
    }
    
    pub fn register_tool(&mut self, tool: Box<dyn MCPTool + Send + Sync>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
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
            r#"You are Karsa MCP Agent. STRICT JSON-ONLY responses.

AVAILABLE TOOLS:
{}

MANDATORY PROTOCOL - Search-Map-Read:
1. SEARCH FIRST: Use 'search' to find relevant files
2. MAP STRUCTURE: Use 'list_symbols' to see functions/classes (NO content read)
3. READ PRECISELY: Use 'file_read_range' for specific lines only

CONTEXT ECONOMY RULES:
- file_read: MAX 300 lines (will REJECT larger files)
- file_read_range: MAX 300 lines per call
- ALWAYS check file size with list_symbols first
- NEVER read entire large files
- Chain multiple range reads if needed

RESPONSE FORMAT (STRICT):
{{
  "thought": "brief reasoning (max 20 words)",
  "tool": "tool_name",
  "params": {{}},
  "next": "continue|done"
}}

WORKFLOW EXAMPLE:
1. {{"thought": "Find auth files", "tool": "search", "params": {{"pattern": "authenticate"}}, "next": "continue"}}
2. {{"thought": "Map auth.rs structure", "tool": "list_symbols", "params": {{"path": "src/auth.rs"}}, "next": "continue"}}
3. {{"thought": "Read login function", "tool": "file_read_range", "params": {{"path": "src/auth.rs", "start_line": 45, "end_line": 80}}, "next": "continue"}}
4. {{"thought": "Task complete", "tool": "none", "params": {{}}, "next": "done"}}

VIOLATIONS = REJECTION:
- Reading files without mapping first
- Requesting >300 lines
- Conversational text in response
- Missing thought/tool/params/next fields"#,
            tools_json
        )
    }
}

// Built-in Tools
struct FileReadTool;
impl MCPTool for FileReadTool {
    fn name(&self) -> &str { "file_read" }
    fn description(&self) -> &str { "Read file contents (max 300 lines, use file_read_range for larger files)" }
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
        
        // Context Economy: Reject large files
        if lines.len() > 300 {
            return Err(anyhow::anyhow!(
                "File too large ({} lines). Use file_read_range(path, start, end) instead. Max 300 lines per read.",
                lines.len()
            ));
        }
        
        Ok(serde_json::json!({ 
            "content": content,
            "lines": lines.len()
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
        
        if end - start > 300 {
            return Err(anyhow::anyhow!("Range too large. Max 300 lines per read."));
        }
        
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        if start < 1 || start > lines.len() || end > lines.len() {
            return Err(anyhow::anyhow!("Invalid line range"));
        }
        
        let range_content = lines[(start-1)..end].join("\n");
        
        Ok(serde_json::json!({ 
            "content": range_content,
            "start": start,
            "end": end,
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
            .output()?;
        
        let results = String::from_utf8_lossy(&output.stdout);
        Ok(serde_json::json!({ "results": results }))
    }
}
