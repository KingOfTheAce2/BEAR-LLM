// MCP (Model Context Protocol) Server for Local Agent Capabilities
// This provides tool-use capabilities for the LLM to act as an autonomous agent

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    pub r#type: String,
    pub properties: HashMap<String, ParameterProperty>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterProperty {
    pub r#type: String,
    pub description: String,
    pub r#enum: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

pub struct MCPServer {
    tools: HashMap<String, Tool>,
    sandboxed: bool,
    allowed_paths: Vec<PathBuf>,
}

impl MCPServer {
    pub fn new(sandboxed: bool) -> Self {
        let mut server = Self {
            tools: HashMap::new(),
            sandboxed,
            allowed_paths: vec![],
        };

        server.register_default_tools();
        server
    }

    fn register_default_tools(&mut self) {
        // File System Tools (sandboxed)
        self.register_tool(Tool {
            name: "read_file".to_string(),
            description: "Read contents of a file".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Path to the file".to_string(),
                        r#enum: None,
                    }),
                ]),
                required: vec!["path".to_string()],
            },
        });

        self.register_tool(Tool {
            name: "write_file".to_string(),
            description: "Write content to a file".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Path to the file".to_string(),
                        r#enum: None,
                    }),
                    ("content".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Content to write".to_string(),
                        r#enum: None,
                    }),
                ]),
                required: vec!["path".to_string(), "content".to_string()],
            },
        });

        self.register_tool(Tool {
            name: "list_directory".to_string(),
            description: "List contents of a directory".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Path to the directory".to_string(),
                        r#enum: None,
                    }),
                ]),
                required: vec!["path".to_string()],
            },
        });

        // Search and Analysis Tools
        self.register_tool(Tool {
            name: "search_documents".to_string(),
            description: "Search through indexed documents".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    ("query".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Search query".to_string(),
                        r#enum: None,
                    }),
                    ("limit".to_string(), ParameterProperty {
                        r#type: "integer".to_string(),
                        description: "Maximum results to return".to_string(),
                        r#enum: None,
                    }),
                ]),
                required: vec!["query".to_string()],
            },
        });

        self.register_tool(Tool {
            name: "extract_text".to_string(),
            description: "Extract text from various file formats".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    ("path".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Path to the file".to_string(),
                        r#enum: None,
                    }),
                    ("format".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "File format".to_string(),
                        r#enum: Some(vec!["pdf".to_string(), "docx".to_string(), "txt".to_string()]),
                    }),
                ]),
                required: vec!["path".to_string()],
            },
        });

        // Legal Document Tools
        self.register_tool(Tool {
            name: "analyze_contract".to_string(),
            description: "Analyze a legal contract for key terms and risks".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    ("content".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Contract text".to_string(),
                        r#enum: None,
                    }),
                    ("type".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Type of contract".to_string(),
                        r#enum: Some(vec![
                            "employment".to_string(),
                            "nda".to_string(),
                            "service".to_string(),
                            "lease".to_string(),
                            "purchase".to_string(),
                        ]),
                    }),
                ]),
                required: vec!["content".to_string()],
            },
        });

        self.register_tool(Tool {
            name: "find_precedents".to_string(),
            description: "Find legal precedents related to a case".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    ("case_description".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Description of the case".to_string(),
                        r#enum: None,
                    }),
                    ("jurisdiction".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Legal jurisdiction".to_string(),
                        r#enum: None,
                    }),
                ]),
                required: vec!["case_description".to_string()],
            },
        });

        // Data Processing Tools
        self.register_tool(Tool {
            name: "execute_sql".to_string(),
            description: "Execute SQL query on local database".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    ("query".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "SQL query (SELECT only in sandbox mode)".to_string(),
                        r#enum: None,
                    }),
                ]),
                required: vec!["query".to_string()],
            },
        });

        self.register_tool(Tool {
            name: "run_python".to_string(),
            description: "Execute Python code in sandboxed environment".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    ("code".to_string(), ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Python code to execute".to_string(),
                        r#enum: None,
                    }),
                    ("imports".to_string(), ParameterProperty {
                        r#type: "array".to_string(),
                        description: "Required Python packages".to_string(),
                        r#enum: None,
                    }),
                ]),
                required: vec!["code".to_string()],
            },
        });
    }

    pub fn register_tool(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.values().cloned().collect()
    }

    pub async fn execute_tool(&self, call: ToolCall) -> Result<ToolResult> {
        match call.tool.as_str() {
            "read_file" => self.handle_read_file(call.parameters).await,
            "write_file" => self.handle_write_file(call.parameters).await,
            "list_directory" => self.handle_list_directory(call.parameters).await,
            "search_documents" => self.handle_search_documents(call.parameters).await,
            "extract_text" => self.handle_extract_text(call.parameters).await,
            "analyze_contract" => self.handle_analyze_contract(call.parameters).await,
            "find_precedents" => self.handle_find_precedents(call.parameters).await,
            "execute_sql" => self.handle_execute_sql(call.parameters).await,
            "run_python" => self.handle_run_python(call.parameters).await,
            _ => Ok(ToolResult {
                success: false,
                result: serde_json::Value::Null,
                error: Some(format!("Unknown tool: {}", call.tool)),
            }),
        }
    }

    async fn handle_read_file(&self, params: serde_json::Value) -> Result<ToolResult> {
        let path = params["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;

        // Security check for sandboxed mode
        if self.sandboxed && !self.is_path_allowed(path) {
            return Ok(ToolResult {
                success: false,
                result: serde_json::Value::Null,
                error: Some("Access denied: Path not in allowed directories".to_string()),
            });
        }

        match fs::read_to_string(path).await {
            Ok(content) => Ok(ToolResult {
                success: true,
                result: serde_json::json!({ "content": content }),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                result: serde_json::Value::Null,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn handle_write_file(&self, params: serde_json::Value) -> Result<ToolResult> {
        let path = params["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
        let content = params["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content parameter"))?;

        // Security check for sandboxed mode
        if self.sandboxed && !self.is_path_allowed(path) {
            return Ok(ToolResult {
                success: false,
                result: serde_json::Value::Null,
                error: Some("Access denied: Path not in allowed directories".to_string()),
            });
        }

        match fs::write(path, content).await {
            Ok(_) => Ok(ToolResult {
                success: true,
                result: serde_json::json!({ "path": path }),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                result: serde_json::Value::Null,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn handle_list_directory(&self, params: serde_json::Value) -> Result<ToolResult> {
        let path = params["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;

        if self.sandboxed && !self.is_path_allowed(path) {
            return Ok(ToolResult {
                success: false,
                result: serde_json::Value::Null,
                error: Some("Access denied: Path not in allowed directories".to_string()),
            });
        }

        match fs::read_dir(path).await {
            Ok(mut entries) => {
                let mut files = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(metadata) = entry.metadata().await {
                        files.push(serde_json::json!({
                            "name": entry.file_name().to_string_lossy(),
                            "is_dir": metadata.is_dir(),
                            "size": metadata.len(),
                        }));
                    }
                }
                Ok(ToolResult {
                    success: true,
                    result: serde_json::json!({ "files": files }),
                    error: None,
                })
            },
            Err(e) => Ok(ToolResult {
                success: false,
                result: serde_json::Value::Null,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn handle_search_documents(&self, params: serde_json::Value) -> Result<ToolResult> {
        let query = params["query"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;
        let limit = params["limit"].as_u64().unwrap_or(10);

        // This would integrate with the RAG engine
        // For now, return a mock response
        Ok(ToolResult {
            success: true,
            result: serde_json::json!({
                "results": [
                    {
                        "title": "Sample Document",
                        "snippet": format!("...content matching '{}'...", query),
                        "relevance": 0.95,
                    }
                ],
                "total": 1,
            }),
            error: None,
        })
    }

    async fn handle_extract_text(&self, _params: serde_json::Value) -> Result<ToolResult> {
        // This would use the file_processor module
        Ok(ToolResult {
            success: true,
            result: serde_json::json!({ "text": "Extracted text content" }),
            error: None,
        })
    }

    async fn handle_analyze_contract(&self, params: serde_json::Value) -> Result<ToolResult> {
        let content = params["content"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content parameter"))?;

        // This would perform actual contract analysis
        // For now, return a structured analysis
        Ok(ToolResult {
            success: true,
            result: serde_json::json!({
                "key_terms": ["Term 1", "Term 2"],
                "risks": ["Risk 1", "Risk 2"],
                "obligations": ["Obligation 1"],
                "dates": ["2024-01-01"],
                "parties": ["Party A", "Party B"],
            }),
            error: None,
        })
    }

    async fn handle_find_precedents(&self, params: serde_json::Value) -> Result<ToolResult> {
        let case_description = params["case_description"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing case_description parameter"))?;

        // This would search a legal database
        Ok(ToolResult {
            success: true,
            result: serde_json::json!({
                "precedents": [
                    {
                        "case_name": "Example v. Sample",
                        "year": "2023",
                        "relevance": 0.85,
                        "summary": "Similar case involving...",
                    }
                ],
            }),
            error: None,
        })
    }

    async fn handle_execute_sql(&self, params: serde_json::Value) -> Result<ToolResult> {
        let query = params["query"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

        // Security: Only allow SELECT in sandbox mode
        if self.sandboxed && !query.trim().to_uppercase().starts_with("SELECT") {
            return Ok(ToolResult {
                success: false,
                result: serde_json::Value::Null,
                error: Some("Only SELECT queries allowed in sandbox mode".to_string()),
            });
        }

        // This would execute against a local SQLite database
        Ok(ToolResult {
            success: true,
            result: serde_json::json!({
                "rows": [],
                "columns": [],
            }),
            error: None,
        })
    }

    async fn handle_run_python(&self, params: serde_json::Value) -> Result<ToolResult> {
        let code = params["code"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing code parameter"))?;

        // Security: This should run in a sandboxed Python environment
        // Using something like RustPython or PyO3 with restrictions

        if self.sandboxed {
            // Check for dangerous imports/functions
            let dangerous = ["os", "subprocess", "eval", "exec", "__import__"];
            for d in dangerous {
                if code.contains(d) {
                    return Ok(ToolResult {
                        success: false,
                        result: serde_json::Value::Null,
                        error: Some(format!("Forbidden operation: {}", d)),
                    });
                }
            }
        }

        // This would execute Python code safely
        Ok(ToolResult {
            success: true,
            result: serde_json::json!({
                "output": "Python execution result",
                "return_value": null,
            }),
            error: None,
        })
    }

    fn is_path_allowed(&self, path: &str) -> bool {
        let path = PathBuf::from(path);

        // Check if path is within allowed directories
        for allowed in &self.allowed_paths {
            if path.starts_with(allowed) {
                return true;
            }
        }

        false
    }

    pub fn add_allowed_path(&mut self, path: PathBuf) {
        self.allowed_paths.push(path);
    }
}

// Agent orchestrator that uses MCP tools
pub struct AgentOrchestrator {
    mcp_server: MCPServer,
}

impl AgentOrchestrator {
    pub fn new(sandboxed: bool) -> Self {
        Self {
            mcp_server: MCPServer::new(sandboxed),
        }
    }

    pub async fn execute_agent_task(&self, task: &str, context: &str) -> Result<String> {
        // This would:
        // 1. Send task to LLM with available tools
        // 2. Parse LLM's tool calls
        // 3. Execute tools via MCP server
        // 4. Return results to LLM
        // 5. Repeat until task complete

        let tools = self.mcp_server.list_tools();
        let tools_json = serde_json::to_string(&tools)?;

        // Format prompt with tools
        let prompt = format!(
            "Task: {}\nContext: {}\nAvailable tools: {}\nExecute the task using the available tools.",
            task, context, tools_json
        );

        // This would interact with the LLM
        Ok("Task completed".to_string())
    }
}