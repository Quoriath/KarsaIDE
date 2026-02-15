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

pub struct MCPCore {
    tools: HashMap<String, Box<dyn MCPTool + Send + Sync>>,
}

pub trait MCPTool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ToolParameter>;
    fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value>;
}

impl MCPCore {
    pub fn new() -> Self {
        let mut core = Self {
            tools: HashMap::new(),
        };
        
        // Register built-in tools
        core.register_tool(Box::new(FileReadTool));
        core.register_tool(Box::new(FileWriteTool));
        core.register_tool(Box::new(SearchTool));
        
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
            r#"You are Karsa MCP Agent. You MUST respond with structured JSON commands only.

AVAILABLE TOOLS:
{}

RESPONSE FORMAT:
{{
  "thought": "brief reasoning",
  "tool": "tool_name",
  "params": {{}},
  "next": "continue|done"
}}

RULES:
- Always use tools for file operations
- Never output conversational text
- Chain multiple tool calls if needed
- Set next="done" when task complete

Example:
{{"thought": "Read file first", "tool": "file_read", "params": {{"path": "src/main.rs"}}, "next": "continue"}}"#,
            tools_json
        )
    }
}

// Built-in Tools
struct FileReadTool;
impl MCPTool for FileReadTool {
    fn name(&self) -> &str { "file_read" }
    fn description(&self) -> &str { "Read file contents" }
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
        Ok(serde_json::json!({ "content": content }))
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
