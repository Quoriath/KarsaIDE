use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

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
    #[serde(default)]
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToolCategory {
    Safe,
    Destructive,
    ReadOnly,
    Write,
    Execute,
}

pub struct MCPCore {
    tools: HashMap<String, Box<dyn MCPTool + Send + Sync>>,
    workspace_path: Option<String>,
}

pub trait MCPTool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ToolParameter>;
    fn required_params(&self) -> Vec<String> {
        self.parameters()
            .iter()
            .filter(|p| p.required)
            .map(|p| p.name.clone())
            .collect()
    }
    fn execute(
        &self,
        params: serde_json::Value,
        workspace: Option<&str>,
    ) -> Result<serde_json::Value>;
    fn category(&self) -> ToolCategory {
        ToolCategory::Safe
    }
}

impl MCPCore {
    pub fn new() -> Self {
        log::info!("Initializing MCP Core...");
        let mut core = Self {
            tools: HashMap::new(),
            workspace_path: None,
        };

        // File operations - Read
        core.register_tool(Box::new(FileReadTool));
        core.register_tool(Box::new(FileReadRangeTool));
        core.register_tool(Box::new(FileReadAroundTool));
        core.register_tool(Box::new(ListFilesTool));
        core.register_tool(Box::new(GetFileInfoTool));
        core.register_tool(Box::new(ListSymbolsTool));
        core.register_tool(Box::new(AnalyzeFileTool));

        // File operations - Write
        core.register_tool(Box::new(FileWriteTool));
        core.register_tool(Box::new(FileAppendTool));
        core.register_tool(Box::new(FileDeleteTool));
        core.register_tool(Box::new(FileMoveTool));
        core.register_tool(Box::new(FileCopyTool));
        core.register_tool(Box::new(CreateDirectoryTool));
        core.register_tool(Box::new(SmartPatchTool));

        // Search
        core.register_tool(Box::new(SearchTool));
        core.register_tool(Box::new(SearchAdvancedTool));
        core.register_tool(Box::new(FindInFileTool));
        core.register_tool(Box::new(ExtractCodeBlockTool));

        // Project
        core.register_tool(Box::new(GetProjectMapTool));
        core.register_tool(Box::new(GetDependenciesTool));

        // Git
        core.register_tool(Box::new(GitStatusTool));
        core.register_tool(Box::new(GitDiffTool));

        log::info!("MCP Core initialized with {} tools", core.tools.len());
        core
    }

    pub fn register_tool(&mut self, tool: Box<dyn MCPTool + Send + Sync>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn set_workspace(&mut self, path: String) {
        self.workspace_path = Some(path);
    }

    pub fn execute(&self, request: MCPRequest) -> MCPResponse {
        match self.tools.get(&request.tool) {
            Some(tool) => match tool.execute(request.params, self.workspace_path.as_deref()) {
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
                error: Some(format!("Unknown tool: {}", request.tool)),
            },
        }
    }

    pub fn execute_with_policy(
        &self,
        request: MCPRequest,
        security: &crate::config_manager::SecuritySettings,
    ) -> MCPResponse {
        match self.tools.get(&request.tool) {
            Some(tool) => {
                let category = tool.category();

                // Check auto-approval based on category
                let auto_approve = match category {
                    ToolCategory::Safe | ToolCategory::ReadOnly => true,
                    ToolCategory::Write => security.auto_save_files,
                    ToolCategory::Destructive => security.auto_delete_files,
                    ToolCategory::Execute => security.auto_execute_shell,
                };

                if auto_approve {
                    match tool.execute(request.params, self.workspace_path.as_deref()) {
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
                error: Some(format!("Unknown tool: {}", request.tool)),
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
                required: tool.required_params(),
            })
            .collect()
    }

    pub fn get_tools_for_prompt(&self) -> String {
        let definitions = self.get_tool_definitions();
        let mut result = String::new();

        for tool in definitions {
            result.push_str(&format!("\n### {}\n{}\n", tool.name, tool.description));

            if !tool.parameters.is_empty() {
                result.push_str("Parameters:\n");
                for param in &tool.parameters {
                    let required = if param.required { " (required)" } else { "" };
                    result.push_str(&format!(
                        "- `{}`: {}{}\n",
                        param.name, param.description, required
                    ));
                }
            }
        }

        result
    }

    pub fn generate_system_prompt(&self, mode: &str, cwd: &str) -> String {
        let mode_config = get_mode_config(mode);
        let tools_section = self.get_tools_for_prompt();

        let cwd_display = if cwd.is_empty() || cwd == "." {
            "No workspace selected. Ask user to open a project folder first."
        } else {
            cwd
        };

        format!(
            r#"{role_definition}

## Tool Usage

{tool_usage_rules}

## Available Tools

{tools_section}

## Environment

- Working directory: {cwd}
- OS: Linux
- Shell: bash

## Mode: {mode_name}

{mode_instructions}

## Response Guidelines

1. Be direct and technical - no flattery or filler words
2. Use tools only when you need file access
3. After getting tool results, answer the user directly
4. If no workspace is set, inform the user to open a project folder
5. Keep responses concise but complete"#,
            role_definition = mode_config.role_definition,
            tool_usage_rules = get_tool_usage_rules(),
            tools_section = tools_section,
            cwd = cwd_display,
            mode_name = mode_config.name,
            mode_instructions = mode_config.instructions,
        )
    }
}

struct ModeConfig {
    name: String,
    role_definition: String,
    instructions: String,
}

fn get_mode_config(mode: &str) -> ModeConfig {
    match mode {
        "code" => ModeConfig {
            name: "Code".to_string(),
            role_definition: r#"You are Karsa, an expert software engineer AI assistant.

CORE PRINCIPLES:
- Be direct, concise, and technical
- Never use flattery ("Great question!", "I'd be happy to help!")
- Never ask follow-up questions unless truly necessary
- Explain complex concepts clearly

COMMUNICATION STYLE:
- Start responses directly with the answer or action
- Use code blocks for code examples
- Be precise and factual
- Acknowledge uncertainty when it exists

When using tools:
1. Briefly mention what you're looking for
2. Execute the tool
3. Analyze results
4. Provide clear answer or next action"#.to_string(),
            instructions: r#"Technical Guidelines:
- Write clean, idiomatic code
- Consider edge cases
- Follow existing project conventions
- Test assumptions by reading relevant files"#.to_string(),
        },
        "architect" => ModeConfig {
            name: "Architect".to_string(),
            role_definition: r#"You are Karsa, a software architect AI.

Focus on:
- System design and architecture decisions
- Technical trade-offs and recommendations
- High-level codebase understanding

Style: Direct, analytical, no flattery."#.to_string(),
            instructions: r#"- Analyze architecture patterns
- Provide clear technical recommendations
- Consider scalability and maintainability"#.to_string(),
        },
        "ask" => ModeConfig {
            name: "Ask".to_string(),
            role_definition: r#"You are Karsa, a programming knowledge assistant.

Answer questions about:
- Programming concepts and best practices
- Code explanations
- Technical guidance

Style: Clear, educational, direct."#.to_string(),
            instructions: r#"- Answer directly and clearly
- Provide code examples when helpful
- Use tools only when you need file context"#.to_string(),
        },
        _ => ModeConfig {
            name: "Code".to_string(),
            role_definition: "You are Karsa, a software engineering AI assistant. Be direct, technical, and helpful.".to_string(),
            instructions: "- Be helpful and direct\n- Use tools when needed".to_string(),
        },
    }
}

fn get_tool_usage_rules() -> String {
    r#"When you need to use a tool, output a JSON array:

[{"name": "tool_name", "arguments": {"param": "value"}}]

Rules:
- Use tools only when you need to read files or search code
- Call tools once, then provide your answer
- Don't repeatedly call the same tools
- For general questions, respond without tools

Examples:
[{"name": "file_read", "arguments": {"path": "src/main.rs"}}]
[{"name": "list_files", "arguments": {"path": "."}}]
[{"name": "search", "arguments": {"pattern": "function", "path": "src"}}]"#
        .to_string()
}

// ============ TOOL IMPLEMENTATIONS ============

struct FileReadTool;
impl MCPTool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }
    fn description(&self) -> &str {
        "Read file contents with optional line range and pattern highlighting."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "File path to read".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "start_line".into(),
                param_type: "number".into(),
                description: "Start line (1-indexed, default: 1)".into(),
                required: false,
                default: Some(serde_json::json!(1)),
                enum_values: None,
            },
            ToolParameter {
                name: "end_line".into(),
                param_type: "number".into(),
                description: "End line (default: 500)".into(),
                required: false,
                default: Some(serde_json::json!(500)),
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();

        let start = params["start_line"].as_u64().unwrap_or(1).max(1) as usize;
        let end = params["end_line"].as_u64().unwrap_or(500).min(total as u64) as usize;

        if start > total {
            return Err(anyhow::anyhow!(
                "start_line {} exceeds file length {}",
                start,
                total
            ));
        }

        let numbered: Vec<String> = lines[(start - 1)..end.min(total)]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:4} | {}", start + i, line))
            .collect();

        Ok(serde_json::json!({
            "content": numbered.join("\n"),
            "start_line": start,
            "end_line": end.min(total),
            "total_lines": total,
            "file_path": path
        }))
    }
}

struct FileReadRangeTool;
impl MCPTool for FileReadRangeTool {
    fn name(&self) -> &str {
        "file_read_range"
    }
    fn description(&self) -> &str {
        "Read a specific line range from a file."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "File path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "start_line".into(),
                param_type: "number".into(),
                description: "Start line (1-indexed)".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "end_line".into(),
                param_type: "number".into(),
                description: "End line (inclusive)".into(),
                required: true,
                default: None,
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        let start = params["start_line"]
            .as_u64()
            .ok_or(anyhow::anyhow!("Missing start_line"))? as usize;
        let end = params["end_line"]
            .as_u64()
            .ok_or(anyhow::anyhow!("Missing end_line"))? as usize;

        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        if start < 1 || start > lines.len() {
            return Err(anyhow::anyhow!("Invalid start_line"));
        }
        if end > lines.len() {
            return Err(anyhow::anyhow!("Invalid end_line"));
        }

        let numbered: Vec<String> = lines[(start - 1)..end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:4} | {}", start + i, line))
            .collect();

        Ok(serde_json::json!({
            "content": numbered.join("\n"),
            "start": start,
            "end": end,
            "file_path": path
        }))
    }
}

struct FileReadAroundTool;
impl MCPTool for FileReadAroundTool {
    fn name(&self) -> &str {
        "file_read_around"
    }
    fn description(&self) -> &str {
        "Read lines around a specific line with context before and after."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "File path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "line".into(),
                param_type: "number".into(),
                description: "Center line number".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "context".into(),
                param_type: "number".into(),
                description: "Lines before/after (default: 10)".into(),
                required: false,
                default: Some(serde_json::json!(10)),
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        let center = params["line"]
            .as_u64()
            .ok_or(anyhow::anyhow!("Missing line"))? as usize;
        let context = params["context"].as_u64().unwrap_or(10) as usize;

        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();

        let start = center.saturating_sub(context + 1).max(1);
        let end = (center + context).min(total);

        let numbered: Vec<String> = lines[(start - 1)..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let ln = start + i;
                if ln == center {
                    format!(">>> {:4} | {}", ln, line)
                } else {
                    format!("    {:4} | {}", ln, line)
                }
            })
            .collect();

        Ok(serde_json::json!({
            "content": numbered.join("\n"),
            "center_line": center,
            "start_line": start,
            "end_line": end,
            "file_path": path
        }))
    }
}

struct ListFilesTool;
impl MCPTool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }
    fn description(&self) -> &str {
        "List files in a directory."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "Directory path (default: .)".into(),
                required: false,
                default: Some(serde_json::json!(".")),
                enum_values: None,
            },
            ToolParameter {
                name: "recursive".into(),
                param_type: "boolean".into(),
                description: "List recursively".into(),
                required: false,
                default: Some(serde_json::json!(false)),
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"].as_str().unwrap_or(".");
        let recursive = params["recursive"].as_bool().unwrap_or(false);

        let mut files = Vec::new();
        let mut dirs = Vec::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path_str = entry.path().display().to_string();
            if entry.file_type()?.is_dir() {
                dirs.push(path_str);
            } else {
                files.push(path_str);
            }
        }

        Ok(serde_json::json!({
            "files": files,
            "directories": dirs,
            "path": path
        }))
    }
}

struct GetFileInfoTool;
impl MCPTool for GetFileInfoTool {
    fn name(&self) -> &str {
        "get_file_info"
    }
    fn description(&self) -> &str {
        "Get file metadata (size, lines, type)."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".into(),
            param_type: "string".into(),
            description: "File path".into(),
            required: true,
            default: None,
            enum_values: None,
        }]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        let metadata = std::fs::metadata(path)?;

        let (lines, file_type) = if metadata.is_file() {
            let content = std::fs::read_to_string(path)?;
            let line_count = content.lines().count();
            let ext = Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown");
            (line_count, ext.to_string())
        } else {
            (0, "directory".to_string())
        };

        Ok(serde_json::json!({
            "path": path,
            "size_bytes": metadata.len(),
            "lines": lines,
            "file_type": file_type,
            "is_file": metadata.is_file(),
            "is_dir": metadata.is_dir()
        }))
    }
}

struct ListSymbolsTool;
impl MCPTool for ListSymbolsTool {
    fn name(&self) -> &str {
        "list_symbols"
    }
    fn description(&self) -> &str {
        "List functions, classes, imports, and exports in a file."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".into(),
            param_type: "string".into(),
            description: "File path".into(),
            required: true,
            default: None,
            enum_values: None,
        }]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        let content = std::fs::read_to_string(path)?;

        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut imports = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let line_num = i + 1;

            // Rust/JS/TS functions
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("function ")
                || trimmed.starts_with("def ")
                || trimmed.starts_with("const ")
            {
                functions.push(json!({"line": line_num, "code": trimmed}));
            }

            // Classes/structs
            if trimmed.starts_with("struct ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("interface ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("impl ")
                || trimmed.starts_with("trait ")
            {
                classes.push(json!({"line": line_num, "code": trimmed}));
            }

            // Imports
            if trimmed.starts_with("use ")
                || trimmed.starts_with("import ")
                || trimmed.starts_with("require(")
                || trimmed.starts_with("#include")
            {
                imports.push(json!({"line": line_num, "code": trimmed}));
            }
        }

        Ok(serde_json::json!({
            "functions": functions,
            "classes": classes,
            "imports": imports,
            "file": path
        }))
    }
}

struct AnalyzeFileTool;
impl MCPTool for AnalyzeFileTool {
    fn name(&self) -> &str {
        "analyze_file"
    }
    fn description(&self) -> &str {
        "Analyze file for complexity, dependencies, and structure."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".into(),
            param_type: "string".into(),
            description: "File path".into(),
            required: true,
            default: None,
            enum_values: None,
        }]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut blank_lines = 0;
        let mut todos = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                blank_lines += 1;
            } else if trimmed.starts_with("//")
                || trimmed.starts_with("#")
                || trimmed.starts_with("/*")
            {
                comment_lines += 1;
                if trimmed.to_lowercase().contains("todo")
                    || trimmed.to_lowercase().contains("fixme")
                {
                    todos.push(json!({"line": i + 1, "text": trimmed}));
                }
            } else {
                code_lines += 1;
            }
        }

        Ok(serde_json::json!({
            "file": path,
            "total_lines": lines.len(),
            "code_lines": code_lines,
            "comment_lines": comment_lines,
            "blank_lines": blank_lines,
            "todos": todos,
            "complexity_score": code_lines / 10
        }))
    }
}

struct FileWriteTool;
impl MCPTool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }
    fn description(&self) -> &str {
        "Write content to a file (creates parent directories if needed)."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "File path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "content".into(),
                param_type: "string".into(),
                description: "File content".into(),
                required: true,
                default: None,
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Write
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        let content = params["content"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing content"))?;

        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;

        Ok(serde_json::json!({ "success": true, "path": path, "bytes_written": content.len() }))
    }
}

struct FileAppendTool;
impl MCPTool for FileAppendTool {
    fn name(&self) -> &str {
        "file_append"
    }
    fn description(&self) -> &str {
        "Append content to an existing file."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "File path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "content".into(),
                param_type: "string".into(),
                description: "Content to append".into(),
                required: true,
                default: None,
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Write
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        let content = params["content"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing content"))?;

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)?;
        file.write_all(content.as_bytes())?;

        Ok(serde_json::json!({ "success": true, "path": path }))
    }
}

struct FileDeleteTool;
impl MCPTool for FileDeleteTool {
    fn name(&self) -> &str {
        "file_delete"
    }
    fn description(&self) -> &str {
        "Delete a file or directory."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".into(),
            param_type: "string".into(),
            description: "Path to delete".into(),
            required: true,
            default: None,
            enum_values: None,
        }]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Destructive
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;

        let metadata = std::fs::metadata(path)?;
        if metadata.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }

        Ok(serde_json::json!({ "success": true, "deleted": path }))
    }
}

struct FileMoveTool;
impl MCPTool for FileMoveTool {
    fn name(&self) -> &str {
        "file_move"
    }
    fn description(&self) -> &str {
        "Move or rename a file or directory."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "from".into(),
                param_type: "string".into(),
                description: "Source path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "to".into(),
                param_type: "string".into(),
                description: "Destination path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Destructive
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let from = params["from"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing from"))?;
        let to = params["to"].as_str().ok_or(anyhow::anyhow!("Missing to"))?;
        std::fs::rename(from, to)?;
        Ok(serde_json::json!({ "success": true, "from": from, "to": to }))
    }
}

struct FileCopyTool;
impl MCPTool for FileCopyTool {
    fn name(&self) -> &str {
        "file_copy"
    }
    fn description(&self) -> &str {
        "Copy a file to a new location."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "from".into(),
                param_type: "string".into(),
                description: "Source path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "to".into(),
                param_type: "string".into(),
                description: "Destination path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Write
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let from = params["from"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing from"))?;
        let to = params["to"].as_str().ok_or(anyhow::anyhow!("Missing to"))?;
        std::fs::copy(from, to)?;
        Ok(serde_json::json!({ "success": true, "from": from, "to": to }))
    }
}

struct CreateDirectoryTool;
impl MCPTool for CreateDirectoryTool {
    fn name(&self) -> &str {
        "create_directory"
    }
    fn description(&self) -> &str {
        "Create a directory and all parent directories."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".into(),
            param_type: "string".into(),
            description: "Directory path to create".into(),
            required: true,
            default: None,
            enum_values: None,
        }]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Write
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        std::fs::create_dir_all(path)?;
        Ok(serde_json::json!({ "success": true, "path": path }))
    }
}

struct SmartPatchTool;
impl MCPTool for SmartPatchTool {
    fn name(&self) -> &str {
        "smart_patch"
    }
    fn description(&self) -> &str {
        "Apply a patch to code with fuzzy matching."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "File path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "find".into(),
                param_type: "string".into(),
                description: "Code to find".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "replace".into(),
                param_type: "string".into(),
                description: "Replacement code".into(),
                required: true,
                default: None,
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Write
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        smart_apply_patch(
            params["path"]
                .as_str()
                .ok_or(anyhow::anyhow!("Missing path"))?,
            params["find"]
                .as_str()
                .ok_or(anyhow::anyhow!("Missing find"))?,
            params["replace"]
                .as_str()
                .ok_or(anyhow::anyhow!("Missing replace"))?,
        )
    }
}

pub fn smart_apply_patch(path: &str, find: &str, replace: &str) -> Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)?;
    let find_trimmed = find.trim();

    if let Some(pos) = content.find(find_trimmed) {
        let new_content = content.replacen(find_trimmed, replace.trim(), 1);
        std::fs::write(path, &new_content)?;
        let line_num = content[..pos].lines().count() + 1;
        return Ok(serde_json::json!({ "success": true, "line": line_num, "method": "exact" }));
    }

    // Fuzzy matching
    let lines: Vec<&str> = content.lines().collect();
    let find_lines: Vec<&str> = find_trimmed.lines().collect();

    for i in 0..lines.len().saturating_sub(find_lines.len()) {
        let matches = find_lines
            .iter()
            .enumerate()
            .filter(|(j, f)| lines[i + j].trim() == f.trim())
            .count();

        if matches > find_lines.len() / 2 {
            let indent = lines[i]
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();
            let replace_lines: Vec<String> = replace
                .trim()
                .lines()
                .map(|l| {
                    if l.trim().is_empty() {
                        String::new()
                    } else {
                        format!("{}{}", indent, l.trim())
                    }
                })
                .collect();

            let mut new_lines = lines[..i].to_vec();
            new_lines.extend(replace_lines.iter().map(|s| s.as_str()));
            new_lines.extend(&lines[i + find_lines.len()..]);

            std::fs::write(path, new_lines.join("\n"))?;
            return Ok(serde_json::json!({ "success": true, "line": i + 1, "method": "fuzzy" }));
        }
    }

    Err(anyhow::anyhow!("Pattern not found"))
}

struct SearchTool;
impl MCPTool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }
    fn description(&self) -> &str {
        "Search for a pattern in files using regex."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "pattern".into(),
                param_type: "string".into(),
                description: "Search pattern (regex)".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "Search path (default: .)".into(),
                required: false,
                default: Some(serde_json::json!(".")),
                enum_values: None,
            },
            ToolParameter {
                name: "file_type".into(),
                param_type: "string".into(),
                description: "File type filter (rs, js, py, etc)".into(),
                required: false,
                default: None,
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let pattern = params["pattern"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing pattern"))?;
        let path = params["path"].as_str().unwrap_or(".");
        let file_type = params["file_type"].as_str();

        let mut cmd = std::process::Command::new("rg");
        cmd.arg("--json").arg("--max-count=50").arg("--smart-case");

        if let Some(ft) = file_type {
            cmd.arg("--type").arg(ft);
        }

        cmd.arg(pattern).arg(path);

        let output = cmd
            .output()
            .map_err(|e| anyhow::anyhow!("ripgrep not found: {}", e))?;

        let results = String::from_utf8_lossy(&output.stdout);
        let mut matches = Vec::new();

        for line in results.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json["type"] == "match" {
                    matches.push(json);
                }
            }
        }

        Ok(serde_json::json!({ "matches": matches, "pattern": pattern }))
    }
}

struct SearchAdvancedTool;
impl MCPTool for SearchAdvancedTool {
    fn name(&self) -> &str {
        "search_advanced"
    }
    fn description(&self) -> &str {
        "Advanced search with include/exclude patterns."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "pattern".into(),
                param_type: "string".into(),
                description: "Search pattern".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "Search path".into(),
                required: false,
                default: Some(serde_json::json!(".")),
                enum_values: None,
            },
            ToolParameter {
                name: "include".into(),
                param_type: "string".into(),
                description: "Include pattern (*.rs)".into(),
                required: false,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "exclude".into(),
                param_type: "string".into(),
                description: "Exclude pattern".into(),
                required: false,
                default: None,
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let pattern = params["pattern"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing pattern"))?;
        let path = params["path"].as_str().unwrap_or(".");
        let include = params["include"].as_str();
        let exclude = params["exclude"].as_str();

        let mut cmd = std::process::Command::new("rg");
        cmd.arg("--json").arg("--max-count=100").arg("--smart-case");

        if let Some(inc) = include {
            cmd.arg("--glob").arg(inc);
        }
        if let Some(exc) = exclude {
            cmd.arg("--glob").arg(format!("!{}", exc));
        }

        cmd.arg(pattern).arg(path);

        let output = cmd.output()?;
        let results = String::from_utf8_lossy(&output.stdout);
        let mut matches = Vec::new();

        for line in results.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json["type"] == "match" {
                    matches.push(json);
                }
            }
        }

        Ok(serde_json::json!({ "matches": matches, "pattern": pattern }))
    }
}

struct FindInFileTool;
impl MCPTool for FindInFileTool {
    fn name(&self) -> &str {
        "find_in_file"
    }
    fn description(&self) -> &str {
        "Find class/function definitions or specific patterns."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "pattern".into(),
                param_type: "string".into(),
                description: "Pattern to find (e.g., 'class UserService', 'fn login')".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "Search path".into(),
                required: false,
                default: Some(serde_json::json!(".")),
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let pattern = params["pattern"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing pattern"))?;
        let path = params["path"].as_str().unwrap_or(".");

        let output = std::process::Command::new("rg")
            .arg("--json")
            .arg("--max-count=30")
            .arg("--context=2")
            .arg(pattern)
            .arg(path)
            .output()?;

        let results = String::from_utf8_lossy(&output.stdout);
        let mut matches = Vec::new();

        for line in results.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if json["type"] == "match" {
                    if let Some(data) = json.get("data") {
                        matches.push(json!({
                            "file": data["path"]["text"],
                            "line": data["line_number"],
                            "code": data["lines"]["text"]
                        }));
                    }
                }
            }
        }

        Ok(serde_json::json!({ "matches": matches, "pattern": pattern }))
    }
}

struct ExtractCodeBlockTool;
impl MCPTool for ExtractCodeBlockTool {
    fn name(&self) -> &str {
        "extract_code_block"
    }
    fn description(&self) -> &str {
        "Extract a specific function or class from a file."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "File path".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ToolParameter {
                name: "block_name".into(),
                param_type: "string".into(),
                description: "Name of function/class to extract".into(),
                required: true,
                default: None,
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing path"))?;
        let block_name = params["block_name"]
            .as_str()
            .ok_or(anyhow::anyhow!("Missing block_name"))?;

        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        let patterns = vec![
            format!(
                r"(pub\s+)?(async\s+)?fn\s+{}\s*[<(]",
                regex::escape(block_name)
            ),
            format!(r"(pub\s+)?struct\s+{}\s*[\{{<]", regex::escape(block_name)),
            format!(r"class\s+{}\s*[\{{<]", regex::escape(block_name)),
            format!(r"def\s+{}\s*\(", regex::escape(block_name)),
        ];

        let mut start: Option<usize> = None;
        for (i, line) in lines.iter().enumerate() {
            for pattern in &patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(line) {
                        start = Some(i);
                        break;
                    }
                }
            }
            if start.is_some() {
                break;
            }
        }

        let start_idx = start.ok_or(anyhow::anyhow!("Block '{}' not found", block_name))?;

        let mut brace_count = 0;
        let mut in_block = false;
        let mut end = start_idx;

        for (i, line) in lines.iter().enumerate().skip(start_idx) {
            for ch in line.chars() {
                match ch {
                    '{' => {
                        brace_count += 1;
                        in_block = true;
                    }
                    '}' => {
                        brace_count -= 1;
                    }
                    _ => {}
                }
            }
            if in_block && brace_count == 0 {
                end = i;
                break;
            }
        }

        let block: Vec<String> = lines[start_idx..=end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:4} | {}", start_idx + i + 1, line))
            .collect();

        Ok(serde_json::json!({
            "block_name": block_name,
            "content": block.join("\n"),
            "start_line": start_idx + 1,
            "end_line": end + 1,
            "file_path": path
        }))
    }
}

struct GetProjectMapTool;
impl MCPTool for GetProjectMapTool {
    fn name(&self) -> &str {
        "get_project_map"
    }
    fn description(&self) -> &str {
        "Get an overview of the project structure."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".into(),
            param_type: "string".into(),
            description: "Project root (default: .)".into(),
            required: false,
            default: Some(serde_json::json!(".")),
            enum_values: None,
        }]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"].as_str().unwrap_or(".");

        let mut file_types: HashMap<String, usize> = HashMap::new();
        let mut entry_points = Vec::new();
        let mut config_files = Vec::new();
        let mut total_files = 0;

        for entry in walkdir::WalkDir::new(path)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.')
                    && name != "node_modules"
                    && name != "target"
                    && name != "dist"
            })
        {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    total_files += 1;

                    let ext = entry
                        .path()
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("other")
                        .to_string();
                    *file_types.entry(ext).or_insert(0) += 1;

                    let name = entry.file_name().to_string_lossy();
                    if name == "main.rs"
                        || name == "index.js"
                        || name == "App.svelte"
                        || name == "main.py"
                    {
                        entry_points.push(entry.path().display().to_string());
                    }
                    if name == "Cargo.toml" || name == "package.json" || name == "go.mod" {
                        config_files.push(entry.path().display().to_string());
                    }
                }
            }
        }

        let mut sorted_types: Vec<_> = file_types.into_iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(serde_json::json!({
            "total_files": total_files,
            "file_types": sorted_types,
            "entry_points": entry_points,
            "config_files": config_files
        }))
    }
}

struct GetDependenciesTool;
impl MCPTool for GetDependenciesTool {
    fn name(&self) -> &str {
        "get_dependencies"
    }
    fn description(&self) -> &str {
        "Get project dependencies from manifest files."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".into(),
            param_type: "string".into(),
            description: "Project root".into(),
            required: false,
            default: Some(serde_json::json!(".")),
            enum_values: None,
        }]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"].as_str().unwrap_or(".");

        let cargo_path = format!("{}/Cargo.toml", path);
        let pkg_path = format!("{}/package.json", path);

        if Path::new(&cargo_path).exists() {
            let content = std::fs::read_to_string(&cargo_path)?;
            let mut deps = Vec::new();

            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(eq_pos) = trimmed.find('=') {
                    let name = trimmed[..eq_pos].trim();
                    if !name.is_empty()
                        && !name.starts_with('[')
                        && !name.starts_with("name")
                        && !name.starts_with("version")
                        && !name.starts_with("edition")
                    {
                        deps.push(name.to_string());
                    }
                }
            }

            return Ok(serde_json::json!({ "type": "rust", "dependencies": deps }));
        }

        if Path::new(&pkg_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let deps: Vec<String> = json
                        .get("dependencies")
                        .and_then(|d| d.as_object())
                        .map(|obj| obj.keys().cloned().collect())
                        .unwrap_or_default();
                    return Ok(serde_json::json!({ "type": "node", "dependencies": deps }));
                }
            }
        }

        Ok(serde_json::json!({ "type": "unknown", "dependencies": [] }))
    }
}

struct GitStatusTool;
impl MCPTool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }
    fn description(&self) -> &str {
        "Get git repository status."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "path".into(),
            param_type: "string".into(),
            description: "Repository path".into(),
            required: false,
            default: Some(serde_json::json!(".")),
            enum_values: None,
        }]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"].as_str().unwrap_or(".");

        let branch_output = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(path)
            .output()?;

        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        let status_output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()?;

        let status = String::from_utf8_lossy(&status_output.stdout);
        let mut staged = Vec::new();
        let mut modified = Vec::new();
        let mut untracked = Vec::new();

        for line in status.lines() {
            if line.len() < 3 {
                continue;
            }
            match &line[..2] {
                "M " | "A " | "R " => staged.push(line[3..].to_string()),
                " M" | "MM" => modified.push(line[3..].to_string()),
                "??" => untracked.push(line[3..].to_string()),
                _ => {}
            }
        }

        Ok(serde_json::json!({
            "branch": branch,
            "staged": staged,
            "modified": modified,
            "untracked": untracked,
            "is_clean": staged.is_empty() && modified.is_empty() && untracked.is_empty()
        }))
    }
}

struct GitDiffTool;
impl MCPTool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }
    fn description(&self) -> &str {
        "Get git diff for staged or unstaged changes."
    }
    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "Repository path".into(),
                required: false,
                default: Some(serde_json::json!(".")),
                enum_values: None,
            },
            ToolParameter {
                name: "staged".into(),
                param_type: "boolean".into(),
                description: "Show staged changes".into(),
                required: false,
                default: Some(serde_json::json!(false)),
                enum_values: None,
            },
        ]
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::ReadOnly
    }
    fn execute(
        &self,
        params: serde_json::Value,
        _workspace: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = params["path"].as_str().unwrap_or(".");
        let staged = params["staged"].as_bool().unwrap_or(false);

        let mut args = vec!["diff"];
        if staged {
            args.push("--staged");
        }

        let output = std::process::Command::new("git")
            .args(&args)
            .current_dir(path)
            .output()?;

        let diff = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(serde_json::json!({
            "diff": diff,
            "has_changes": !diff.is_empty()
        }))
    }
}
