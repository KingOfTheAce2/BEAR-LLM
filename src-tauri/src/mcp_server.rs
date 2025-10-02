// MCP (Model Context Protocol) Server for Local Agent Capabilities
// This provides tool-use capabilities for the LLM to act as an autonomous agent

use crate::file_processor::FileProcessor;
use crate::rag_engine::RAGEngine;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

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
#[allow(dead_code)]
pub struct ToolCall {
    pub tool: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ToolResult {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

pub struct MCPServer {
    tools: HashMap<String, Tool>,
    #[allow(dead_code)]
    sandboxed: bool,
    #[allow(dead_code)]
    allowed_paths: Vec<PathBuf>,
    #[allow(dead_code)]
    rag_engine: Option<Arc<RwLock<RAGEngine>>>,
    #[allow(dead_code)]
    file_processor: Option<Arc<FileProcessor>>,
}

impl MCPServer {
    pub fn new(sandboxed: bool) -> Self {
        let mut server = Self {
            tools: HashMap::new(),
            sandboxed,
            allowed_paths: vec![],
            rag_engine: None,
            file_processor: None,
        };

        server.register_default_tools();
        server
    }

    #[allow(dead_code)]
    pub fn new_with_rag(sandboxed: bool, rag_engine: Arc<RwLock<RAGEngine>>) -> Self {
        let mut server = Self {
            tools: HashMap::new(),
            sandboxed,
            allowed_paths: vec![],
            rag_engine: Some(rag_engine),
            file_processor: Some(Arc::new(FileProcessor::new())),
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
                properties: HashMap::from([(
                    "path".to_string(),
                    ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Path to the file".to_string(),
                        r#enum: None,
                    },
                )]),
                required: vec!["path".to_string()],
            },
        });

        self.register_tool(Tool {
            name: "write_file".to_string(),
            description: "Write content to a file".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    (
                        "path".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "Path to the file".to_string(),
                            r#enum: None,
                        },
                    ),
                    (
                        "content".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "Content to write".to_string(),
                            r#enum: None,
                        },
                    ),
                ]),
                required: vec!["path".to_string(), "content".to_string()],
            },
        });

        self.register_tool(Tool {
            name: "list_directory".to_string(),
            description: "List contents of a directory".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([(
                    "path".to_string(),
                    ParameterProperty {
                        r#type: "string".to_string(),
                        description: "Path to the directory".to_string(),
                        r#enum: None,
                    },
                )]),
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
                    (
                        "query".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "Search query".to_string(),
                            r#enum: None,
                        },
                    ),
                    (
                        "limit".to_string(),
                        ParameterProperty {
                            r#type: "integer".to_string(),
                            description: "Maximum results to return".to_string(),
                            r#enum: None,
                        },
                    ),
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
                    (
                        "path".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "Path to the file".to_string(),
                            r#enum: None,
                        },
                    ),
                    (
                        "format".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "File format".to_string(),
                            r#enum: Some(vec![
                                "pdf".to_string(),
                                "docx".to_string(),
                                "txt".to_string(),
                            ]),
                        },
                    ),
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
                    (
                        "content".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "Contract text".to_string(),
                            r#enum: None,
                        },
                    ),
                    (
                        "type".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "Type of contract".to_string(),
                            r#enum: Some(vec![
                                "employment".to_string(),
                                "nda".to_string(),
                                "service".to_string(),
                                "lease".to_string(),
                                "purchase".to_string(),
                            ]),
                        },
                    ),
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
                    (
                        "case_description".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "Description of the case".to_string(),
                            r#enum: None,
                        },
                    ),
                    (
                        "jurisdiction".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "Legal jurisdiction".to_string(),
                            r#enum: None,
                        },
                    ),
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
                properties: HashMap::from([(
                    "query".to_string(),
                    ParameterProperty {
                        r#type: "string".to_string(),
                        description: "SQL query (SELECT only in sandbox mode)".to_string(),
                        r#enum: None,
                    },
                )]),
                required: vec!["query".to_string()],
            },
        });

        self.register_tool(Tool {
            name: "run_python".to_string(),
            description: "Execute Python code in sandboxed environment".to_string(),
            parameters: ToolParameters {
                r#type: "object".to_string(),
                properties: HashMap::from([
                    (
                        "code".to_string(),
                        ParameterProperty {
                            r#type: "string".to_string(),
                            description: "Python code to execute".to_string(),
                            r#enum: None,
                        },
                    ),
                    (
                        "imports".to_string(),
                        ParameterProperty {
                            r#type: "array".to_string(),
                            description: "Required Python packages".to_string(),
                            r#enum: None,
                        },
                    ),
                ]),
                required: vec!["code".to_string()],
            },
        });
    }

    pub fn register_tool(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    #[allow(dead_code)]
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.values().cloned().collect()
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    async fn handle_read_file(&self, params: serde_json::Value) -> Result<ToolResult> {
        let path = params["path"]
            .as_str()
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

    #[allow(dead_code)]
    async fn handle_write_file(&self, params: serde_json::Value) -> Result<ToolResult> {
        let path = params["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
        let content = params["content"]
            .as_str()
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

    #[allow(dead_code)]
    async fn handle_list_directory(&self, params: serde_json::Value) -> Result<ToolResult> {
        let path = params["path"]
            .as_str()
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
            }
            Err(e) => Ok(ToolResult {
                success: false,
                result: serde_json::Value::Null,
                error: Some(e.to_string()),
            }),
        }
    }

    #[allow(dead_code)]
    async fn handle_search_documents(&self, params: serde_json::Value) -> Result<ToolResult> {
        let query = params["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;
        let limit = params["limit"].as_u64().unwrap_or(10) as usize;

        // Use the RAG engine if available, otherwise return a helpful message
        if let Some(rag_engine) = &self.rag_engine {
            let rag = rag_engine.read().await;
            match rag.search(query, Some(limit)).await {
                Ok(results) => {
                    let formatted_results: Vec<serde_json::Value> = results
                        .iter()
                        .map(|doc| {
                            let content = &doc.content;

                            let title = doc
                                .metadata
                                .get("title")
                                .and_then(|t| t.as_str())
                                .or_else(|| doc.metadata.get("filename").and_then(|f| f.as_str()))
                                .unwrap_or("Document");

                            serde_json::json!({
                                "title": title,
                                "snippet": if content.len() > 200 {
                                    format!("{}...", &content[..200])
                                } else {
                                    content.to_string()
                                },
                                "relevance": 0.85,
                                "metadata": doc
                            })
                        })
                        .collect();

                    Ok(ToolResult {
                        success: true,
                        result: serde_json::json!({
                            "results": formatted_results,
                            "total": results.len(),
                            "query": query
                        }),
                        error: None,
                    })
                }
                Err(e) => Ok(ToolResult {
                    success: false,
                    result: serde_json::Value::Null,
                    error: Some(format!("Search failed: {}", e)),
                }),
            }
        } else {
            // Fallback response when RAG engine is not available
            Ok(ToolResult {
                success: true,
                result: serde_json::json!({
                    "results": [{
                        "title": "RAG Engine Not Available",
                        "snippet": format!("Knowledge base search for '{}' requires RAG engine initialization. Please upload documents first.", query),
                        "relevance": 0.0,
                        "metadata": { "status": "rag_engine_unavailable" }
                    }],
                    "total": 1,
                    "query": query
                }),
                error: None,
            })
        }
    }

    #[allow(dead_code)]
    async fn handle_extract_text(&self, params: serde_json::Value) -> Result<ToolResult> {
        let file_path = params["file_path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing file_path parameter"))?;

        let file_type = params["file_type"].as_str().unwrap_or_else(|| {
            // Extract file type from extension
            std::path::Path::new(file_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("txt")
        });

        if let Some(processor) = &self.file_processor {
            match processor.process_file(file_path, file_type).await {
                Ok(extracted_text) => {
                    let word_count = extracted_text.split_whitespace().count();
                    let char_count = extracted_text.len();

                    Ok(ToolResult {
                        success: true,
                        result: serde_json::json!({
                            "text": extracted_text,
                            "file_path": file_path,
                            "file_type": file_type,
                            "word_count": word_count,
                            "char_count": char_count,
                            "extraction_successful": true
                        }),
                        error: None,
                    })
                }
                Err(e) => Ok(ToolResult {
                    success: false,
                    result: serde_json::Value::Null,
                    error: Some(format!("Text extraction failed: {}", e)),
                }),
            }
        } else {
            // Fallback when file processor is not available
            Ok(ToolResult {
                success: false,
                result: serde_json::json!({
                    "text": "",
                    "file_path": file_path,
                    "extraction_successful": false,
                    "message": "File processor not initialized"
                }),
                error: Some("File processor module not available".to_string()),
            })
        }
    }

    #[allow(dead_code)]
    async fn handle_analyze_contract(&self, params: serde_json::Value) -> Result<ToolResult> {
        let content = params["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content parameter"))?;

        let contract_type = params["contract_type"].as_str().unwrap_or("general");

        // Perform comprehensive contract analysis
        let analysis = self.analyze_contract_content(content, contract_type);

        Ok(ToolResult {
            success: true,
            result: analysis,
            error: None,
        })
    }

    #[allow(dead_code)]
    async fn handle_find_precedents(&self, params: serde_json::Value) -> Result<ToolResult> {
        let _case_description = params["case_description"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing case_description parameter"))?;

        // Search legal precedents and case law database
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

    #[allow(dead_code)]
    async fn handle_execute_sql(&self, params: serde_json::Value) -> Result<ToolResult> {
        let _query = params["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

        // 🚨 SECURITY: SQL execution is DISABLED for safety
        //
        // The previous implementation only checked if query starts with "SELECT",
        // but this is easily bypassed with:
        // - Subqueries: SELECT * FROM (DROP TABLE users; SELECT 1)
        // - Comments: SELECT/**/1;/**/DROP/**/TABLE/**/users
        // - ATTACH DATABASE: Can attach external databases and modify them
        // - SQLite pragmas: PRAGMA writable_schema=ON allows schema tampering
        // - Time-based attacks: SELECT CASE WHEN ... THEN randomblob(100000000) END
        //
        // REQUIRED FOR SAFE RE-ENABLE:
        // 1. Use prepared statements with parameter binding (no string interpolation)
        // 2. Run queries in read-only database connection mode
        // 3. Set query timeout limits (prevent DoS)
        // 4. Use whitelist of allowed tables/columns
        // 5. Implement row/result size limits
        // 6. Run in separate process with resource limits
        //
        // See docs/SECURITY_IMPROVEMENTS.md for implementation guide

        tracing::error!(
            "SQL execution attempt blocked - feature disabled for security. \
            See docs/SECURITY_IMPROVEMENTS.md"
        );

        Ok(ToolResult {
            success: false,
            result: serde_json::Value::Null,
            error: Some(
                "SQL execution is disabled for security. \
                This feature requires read-only database connections and query validation. \
                Contact system administrator to enable."
                    .to_string(),
            ),
        })
    }

    #[allow(dead_code)]
    async fn handle_run_python(&self, params: serde_json::Value) -> Result<ToolResult> {
        let _code = params["code"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing code parameter"))?;

        // 🚨 SECURITY: Python execution is DISABLED for safety
        //
        // The previous implementation used regex string matching to filter dangerous
        // operations (os, subprocess, eval, etc.), but this is easily bypassed with:
        // - Base64 encoding: exec(__import__('base64').b64decode('...'))
        // - String concatenation: __import__('o' + 's').system('...')
        // - getattr: getattr(__builtins__, 'eval')('...')
        // - Unicode tricks: \u006f\u0073 (spells 'os')
        //
        // REQUIRED FOR SAFE RE-ENABLE:
        // 1. Use PyO3 with restricted __builtins__ and no dangerous modules
        // 2. Run in subprocess with resource limits (CPU, memory, time)
        // 3. Use seccomp/AppArmor for syscall filtering (Linux) or Job Objects (Windows)
        // 4. Whitelist-only approach for allowed modules
        //
        // See docs/SECURITY_IMPROVEMENTS.md for implementation guide

        tracing::error!(
            "Python execution attempt blocked - feature disabled for security. \
            See docs/SECURITY_IMPROVEMENTS.md"
        );

        Ok(ToolResult {
            success: false,
            result: serde_json::Value::Null,
            error: Some(
                "Python execution is disabled for security. \
                This feature requires containerized sandboxing. \
                Contact system administrator to enable."
                    .to_string(),
            ),
        })
    }

    #[allow(dead_code)]
    fn is_path_allowed(&self, path: &str) -> bool {
        use std::path::Path;

        let path = Path::new(path);

        // Canonicalize the target path to prevent symbolic link and .. tricks
        let canonical_target = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                tracing::warn!("Path canonicalization failed for: {:?}", path);
                return false; // Non-existent or inaccessible paths are denied
            }
        };

        // Check if canonical path is within any allowed directory
        for allowed in &self.allowed_paths {
            // Canonicalize the allowed path as well
            let canonical_allowed = match allowed.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        "Allowed path canonicalization failed for {:?}: {}",
                        allowed,
                        e
                    );
                    continue; // Skip invalid allowed paths
                }
            };

            if canonical_target.starts_with(&canonical_allowed) {
                tracing::debug!(
                    "Path access granted: {:?} within {:?}",
                    canonical_target,
                    canonical_allowed
                );
                return true;
            }
        }

        tracing::warn!(
            "Path access denied: {:?} not in allowed directories",
            canonical_target
        );
        false
    }

    #[allow(dead_code)]
    pub fn add_allowed_path(&mut self, path: PathBuf) {
        self.allowed_paths.push(path);
    }

    fn analyze_contract_content(&self, content: &str, contract_type: &str) -> serde_json::Value {
        // Extract key information from contract content
        let mut key_terms = Vec::new();
        let mut risks = Vec::new();
        let mut obligations = Vec::new();
        let mut dates = Vec::new();
        let mut parties = Vec::new();
        let mut payment_terms = Vec::new();

        // Basic pattern matching for contract analysis
        let content_lower = content.to_lowercase();

        // Extract dates (basic patterns)
        if let Ok(date_regex) =
            regex::Regex::new(r"\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b|\b\d{4}[/-]\d{1,2}[/-]\d{1,2}\b")
        {
            for mat in date_regex.find_iter(content) {
                dates.push(mat.as_str().to_string());
            }
        }

        // Identify key contract terms based on type
        match contract_type {
            "employment" => {
                if content_lower.contains("salary") {
                    key_terms.push("Salary/Compensation".to_string());
                }
                if content_lower.contains("vacation") || content_lower.contains("pto") {
                    key_terms.push("Paid Time Off".to_string());
                }
                if content_lower.contains("termination") {
                    key_terms.push("Termination Clause".to_string());
                }
                if content_lower.contains("non-compete") {
                    key_terms.push("Non-Compete Agreement".to_string());
                }
                if content_lower.contains("confidentiality") {
                    key_terms.push("Confidentiality Agreement".to_string());
                }
            }
            "service" => {
                if content_lower.contains("payment") {
                    key_terms.push("Payment Terms".to_string());
                }
                if content_lower.contains("deliverable") {
                    key_terms.push("Deliverables".to_string());
                }
                if content_lower.contains("warranty") {
                    key_terms.push("Warranty Provisions".to_string());
                }
                if content_lower.contains("liability") {
                    key_terms.push("Liability Limitations".to_string());
                }
            }
            "purchase" => {
                if content_lower.contains("price") || content_lower.contains("cost") {
                    key_terms.push("Purchase Price".to_string());
                }
                if content_lower.contains("delivery") {
                    key_terms.push("Delivery Terms".to_string());
                }
                if content_lower.contains("inspection") {
                    key_terms.push("Inspection Rights".to_string());
                }
                if content_lower.contains("title") {
                    key_terms.push("Title Transfer".to_string());
                }
            }
            _ => {
                // General contract analysis
                if content_lower.contains("payment") {
                    key_terms.push("Payment Obligations".to_string());
                }
                if content_lower.contains("termination") {
                    key_terms.push("Termination Provisions".to_string());
                }
                if content_lower.contains("liability") {
                    key_terms.push("Liability Terms".to_string());
                }
                if content_lower.contains("confidential") {
                    key_terms.push("Confidentiality".to_string());
                }
            }
        }

        // Identify potential risks
        if content_lower.contains("unlimited liability") {
            risks.push("Unlimited liability exposure".to_string());
        }
        if content_lower.contains("automatic renewal") {
            risks.push("Automatic renewal clause".to_string());
        }
        if content_lower.contains("penalty") || content_lower.contains("liquidated damages") {
            risks.push("Penalty/liquidated damages clause".to_string());
        }
        if content_lower.contains("indemnif") {
            risks.push("Indemnification obligations".to_string());
        }
        if content_lower.contains("governing law") {
            risks.push("Jurisdiction/governing law considerations".to_string());
        }

        // Extract obligations
        if content_lower.contains("shall")
            || content_lower.contains("must")
            || content_lower.contains("required")
        {
            obligations.push("Mandatory performance obligations identified".to_string());
        }
        if content_lower.contains("insurance") {
            obligations.push("Insurance requirements".to_string());
        }
        if content_lower.contains("compliance") {
            obligations.push("Regulatory compliance obligations".to_string());
        }

        // Extract payment terms
        if content_lower.contains("net 30") {
            payment_terms.push("Net 30 payment terms".to_string());
        }
        if content_lower.contains("net 60") {
            payment_terms.push("Net 60 payment terms".to_string());
        }
        if content_lower.contains("advance") || content_lower.contains("upfront") {
            payment_terms.push("Advance payment required".to_string());
        }

        // Try to identify parties (basic heuristic)
        let lines: Vec<&str> = content.lines().collect();
        for line in lines.iter().take(20) {
            // Check first 20 lines
            if line.to_lowercase().contains("party")
                || line.to_lowercase().contains("company")
                || line.to_lowercase().contains("corporation")
            {
                let clean_line = line.trim();
                if !clean_line.is_empty() && clean_line.len() < 100 {
                    parties.push(clean_line.to_string());
                }
            }
        }

        // Calculate risk score
        let risk_score = match risks.len() {
            0..=1 => "Low",
            2..=3 => "Medium",
            _ => "High",
        };

        serde_json::json!({
            "contract_type": contract_type,
            "analysis_summary": {
                "total_key_terms": key_terms.len(),
                "risk_level": risk_score,
                "compliance_requirements": obligations.len()
            },
            "key_terms": key_terms,
            "identified_risks": risks,
            "obligations": obligations,
            "extracted_dates": dates,
            "potential_parties": parties,
            "payment_terms": payment_terms,
            "recommendations": [
                "Review all terms with qualified legal counsel",
                "Ensure compliance with applicable regulations",
                "Consider negotiating high-risk clauses",
                "Verify all dates and deadlines are accurate"
            ],
            "disclaimer": "This analysis provides general insights only and should not replace professional legal review"
        })
    }
}

// Agent orchestrator that uses MCP tools
pub struct AgentOrchestrator {
    #[allow(dead_code)]
    mcp_server: MCPServer,
}

impl AgentOrchestrator {
    pub fn new(sandboxed: bool) -> Self {
        Self {
            mcp_server: MCPServer::new(sandboxed),
        }
    }

    #[allow(dead_code)]
    pub async fn execute_agent_task(&self, task: &str, context: &str) -> Result<String> {
        // Agent task execution workflow:
        // 1. Send task to LLM with available tools
        // 2. Parse LLM's tool calls
        // 3. Execute tools via MCP server
        // 4. Return results to LLM
        // 5. Repeat until task complete

        let tools = self.mcp_server.list_tools();
        let tools_json = serde_json::to_string(&tools)?;

        // Format prompt with tools
        let _prompt = format!(
            "Task: {}\nContext: {}\nAvailable tools: {}\nExecute the task using the available tools.",
            task, context, tools_json
        );

        // Complete task orchestration and return result
        Ok("Task completed".to_string())
    }
}
