//! MCP (Model Context Protocol) tool management.

use crate::error::{Result, VaultError};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// MCP (Model Context Protocol) Tool Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Input schema (JSON Schema)
    pub input_schema: JsonValue,

    /// Tool metadata
    pub metadata: HashMap<String, String>,
}

impl MCPTool {
    /// Create a new MCP tool
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            metadata: HashMap::new(),
        }
    }

    /// Add a parameter to the tool schema
    pub fn add_parameter(
        mut self,
        name: &str,
        param_type: &str,
        description: &str,
        required: bool,
    ) -> Self {
        if let Some(properties) = self.input_schema.get_mut("properties") {
            if let Some(props) = properties.as_object_mut() {
                props.insert(
                    name.to_string(),
                    serde_json::json!({
                        "type": param_type,
                        "description": description
                    }),
                );
            }
        }

        if required {
            if let Some(required_array) = self.input_schema.get_mut("required") {
                if let Some(arr) = required_array.as_array_mut() {
                    arr.push(serde_json::json!(name));
                }
            }
        }

        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Tool execution context
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Document store reference
    pub document_store: Option<String>,

    /// Knowledge base reference
    pub knowledge_base: Option<String>,

    /// Additional context data
    pub data: HashMap<String, String>,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new() -> Self {
        Self {
            document_store: None,
            knowledge_base: None,
            data: HashMap::new(),
        }
    }

    /// Set document store reference
    pub fn with_document_store(mut self, store_id: String) -> Self {
        self.document_store = Some(store_id);
        self
    }

    /// Set knowledge base reference
    pub fn with_knowledge_base(mut self, kb_id: String) -> Self {
        self.knowledge_base = Some(kb_id);
        self
    }

    /// Add context data
    pub fn with_data(mut self, key: String, value: String) -> Self {
        self.data.insert(key, value);
        self
    }
}

impl Default for ToolContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Success status
    pub success: bool,

    /// Result data (JSON)
    pub data: JsonValue,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Execution metadata
    pub metadata: HashMap<String, String>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(data: JsonValue) -> Self {
        Self {
            success: true,
            data,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a failed result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            data: serde_json::json!(null),
            error: Some(error),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Tool executor trait
pub trait ToolExecutor {
    /// Execute a tool with given parameters
    fn execute(
        &self,
        tool_name: &str,
        params: JsonValue,
        context: &ToolContext,
    ) -> Result<ToolResult>;

    /// List available tools
    fn list_tools(&self) -> Vec<MCPTool>;

    /// Get tool by name
    fn get_tool(&self, name: &str) -> Option<&MCPTool>;
}

/// Type alias for tool executor functions
type ToolExecutorFn = Box<dyn Fn(JsonValue, &ToolContext) -> Result<ToolResult>>;

/// MCP Server for tool management
pub struct MCPServer {
    tools: HashMap<String, MCPTool>,
    executors: HashMap<String, ToolExecutorFn>,
}

impl MCPServer {
    /// Create a new MCP server
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            executors: HashMap::new(),
        }
    }

    /// Register a tool with executor function
    pub fn register_tool<F>(&mut self, tool: MCPTool, executor: F) -> Result<()>
    where
        F: Fn(JsonValue, &ToolContext) -> Result<ToolResult> + 'static,
    {
        let name = tool.name.clone();
        self.tools.insert(name.clone(), tool);
        self.executors.insert(name, Box::new(executor));
        Ok(())
    }

    /// Execute a tool
    pub fn execute_tool(
        &self,
        tool_name: &str,
        params: JsonValue,
        context: &ToolContext,
    ) -> Result<ToolResult> {
        let executor = self
            .executors
            .get(tool_name)
            .ok_or_else(|| VaultError::InvalidInput(format!("Tool '{}' not found", tool_name)))?;

        executor(params, context)
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<&MCPTool> {
        self.tools.values().collect()
    }

    /// Get tool by name
    pub fn get_tool(&self, name: &str) -> Option<&MCPTool> {
        self.tools.get(name)
    }

    /// Register built-in RAG tools
    pub fn register_builtin_tools(&mut self) -> Result<()> {
        // Tool: Search documents
        let search_tool = MCPTool::new(
            "search_documents".to_string(),
            "Search for similar documents using vector embeddings".to_string(),
        )
        .add_parameter("query", "string", "Search query text", true)
        .add_parameter("top_k", "number", "Number of results to return", false)
        .add_parameter("threshold", "number", "Minimum similarity threshold", false);

        self.register_tool(search_tool, |params, _ctx| {
            let query = params
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| VaultError::InvalidInput("Missing 'query' parameter".to_string()))?;

            let top_k = params.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

            let _threshold = params
                .get("threshold")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5);

            // NOTE: MCP tool closures are stateless — actual search requires a DocumentStore
            // or database connection injected via ToolContext. Without it, we return an honest
            // response indicating the tool was invoked but needs integration with a backing store.
            Ok(ToolResult::success(serde_json::json!({
                "query": query,
                "top_k": top_k,
                "results": [],
                "note": "Connect a DocumentStore or SQLiteDatabase via ToolContext for live search results"
            })))
        })?;

        // Tool: Add document
        let add_doc_tool = MCPTool::new(
            "add_document".to_string(),
            "Add a document to the knowledge base".to_string(),
        )
        .add_parameter("id", "string", "Document ID", true)
        .add_parameter("content", "string", "Document content", true)
        .add_parameter("metadata", "object", "Document metadata", false);

        self.register_tool(add_doc_tool, |params, _ctx| {
            let id = params
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| VaultError::InvalidInput("Missing 'id' parameter".to_string()))?;

            let content = params
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    VaultError::InvalidInput("Missing 'content' parameter".to_string())
                })?;

            let metadata = params
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            Ok(ToolResult::success(serde_json::json!({
                "id": id,
                "content_length": content.len(),
                "metadata": metadata,
                "status": "accepted",
                "note": "Connect a DocumentStore or SQLiteDatabase via ToolContext for persistent storage"
            })))
        })?;

        // Tool: Chunk text
        let chunk_tool = MCPTool::new(
            "chunk_text".to_string(),
            "Split text into chunks for processing".to_string(),
        )
        .add_parameter("text", "string", "Text to chunk", true)
        .add_parameter("chunk_size", "number", "Size of each chunk", false)
        .add_parameter("overlap", "number", "Overlap between chunks", false);

        self.register_tool(chunk_tool, |params, _ctx| {
            let text = params
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| VaultError::InvalidInput("Missing 'text' parameter".to_string()))?;

            let chunk_size = params
                .get("chunk_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(512) as usize;

            let overlap = params.get("overlap").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

            // Actually split text into chunks
            let chars: Vec<char> = text.chars().collect();
            let mut chunks = Vec::new();
            let mut start = 0;

            while start < chars.len() {
                let end = (start + chunk_size).min(chars.len());
                let chunk_text: String = chars[start..end].iter().collect();
                chunks.push(serde_json::json!({
                    "index": chunks.len(),
                    "start": start,
                    "end": end,
                    "text": chunk_text
                }));

                if end >= chars.len() {
                    break;
                }
                start = end - overlap.min(end);
            }

            Ok(ToolResult::success(serde_json::json!({
                "text_length": text.len(),
                "chunk_size": chunk_size,
                "overlap": overlap,
                "num_chunks": chunks.len(),
                "chunks": chunks
            })))
        })?;

        // Tool: Execute rule
        let rule_tool = MCPTool::new(
            "execute_rule".to_string(),
            "Execute a business rule with given context".to_string(),
        )
        .add_parameter("rule_id", "string", "Rule identifier", true)
        .add_parameter("context", "object", "Rule execution context", true);

        self.register_tool(rule_tool, |params, _ctx| {
            let rule_id = params
                .get("rule_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    VaultError::InvalidInput("Missing 'rule_id' parameter".to_string())
                })?;

            let context = params
                .get("context")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            // NOTE: Rule execution requires a RuleEngine instance with registered rules.
            // The MCP tool accepts the request but needs integration via ToolContext.
            Ok(ToolResult::success(serde_json::json!({
                "rule_id": rule_id,
                "context_received": context,
                "status": "accepted",
                "note": "Connect a RuleEngine via ToolContext for live rule execution"
            })))
        })?;

        Ok(())
    }
}

impl Default for MCPServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_success_and_failure() {
        let ok = ToolResult::success(serde_json::json!({"key": "val"}));
        assert!(ok.success);
        assert!(ok.error.is_none());

        let err = ToolResult::failure("bad input".to_string());
        assert!(!err.success);
        assert_eq!(err.error.as_deref(), Some("bad input"));
    }

    #[test]
    fn test_tool_result_with_metadata() {
        let tr = ToolResult::success(serde_json::json!("data"))
            .with_metadata("k".to_string(), "v".to_string());
        assert!(!tr.metadata.is_empty());
        assert_eq!(tr.metadata["k"], "v");
    }

    #[test]
    fn test_mcp_tool_builder() {
        let tool = MCPTool::new("search".to_string(), "Search docs".to_string())
            .add_parameter("query", "string", "The query", true)
            .add_parameter("limit", "integer", "Max results", false)
            .with_metadata("version".to_string(), "1.0".to_string());
        assert_eq!(tool.name, "search");
        assert_eq!(tool.input_schema["properties"]["query"]["type"], "string");
        assert!(tool.input_schema["required"]
            .as_array()
            .unwrap()
            .contains(&"query".into()));
        assert!(!tool.metadata.is_empty());
    }

    #[test]
    fn test_tool_context_builder() {
        let ctx = ToolContext::new()
            .with_data("key1".to_string(), "value1".to_string())
            .with_data("key2".to_string(), "42".to_string());
        assert_eq!(ctx.data["key1"], "value1");
        assert_eq!(ctx.data["key2"], "42");
    }

    #[test]
    fn test_mcp_server_register_and_list() {
        let mut server = MCPServer::new();
        let tool = MCPTool::new("test_tool".to_string(), "A test tool".to_string());
        server
            .register_tool(tool, |_params, _ctx| {
                Ok(ToolResult::success(serde_json::json!("done")))
            })
            .unwrap();

        let tools = server.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");

        let retrieved = server.get_tool("test_tool").unwrap();
        assert_eq!(retrieved.name, "test_tool");
    }

    #[test]
    fn test_mcp_server_execute_tool() {
        let mut server = MCPServer::new();
        let tool = MCPTool::new("echo".to_string(), "Echo tool".to_string());
        server
            .register_tool(tool, |params, _ctx| {
                let msg = params.get("msg").and_then(|v| v.as_str()).unwrap_or("none");
                Ok(ToolResult::success(serde_json::json!({"echo": msg})))
            })
            .unwrap();

        let ctx = ToolContext::new();
        let params = serde_json::json!({"msg": "hello"});
        let result = server.execute_tool("echo", params, &ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.data["echo"], "hello");
    }

    #[test]
    fn test_mcp_server_execute_nonexistent() {
        let server = MCPServer::new();
        let ctx = ToolContext::new();
        let result = server.execute_tool("ghost", serde_json::json!({}), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_server_register_builtins() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let tools = server.list_tools();
        assert!(tools.len() >= 4);

        // Verify builtin tool names
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"search_documents"));
        assert!(names.contains(&"add_document"));
        assert!(names.contains(&"chunk_text"));
        assert!(names.contains(&"execute_rule"));
    }

    #[test]
    fn test_builtin_chunk_text() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let ctx = ToolContext::new();
        let params = serde_json::json!({
            "text": "Hello world. This is a test. Another sentence here.",
            "chunk_size": 20,
            "overlap": 5,
        });
        let result = server.execute_tool("chunk_text", params, &ctx).unwrap();
        assert!(result.success);
        let chunks = result.data["chunks"].as_array().unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_builtin_execute_rule() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let ctx = ToolContext::new();
        let params = serde_json::json!({
            "rule_id": "test_rule",
            "context": {"key": "value"},
        });
        let result = server.execute_tool("execute_rule", params, &ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.data["rule_id"], "test_rule");
        assert_eq!(result.data["status"], "accepted");
    }

    #[test]
    fn test_builtin_search_documents() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let ctx = ToolContext::new();
        let params = serde_json::json!({
            "query": "test query",
            "top_k": 5,
        });
        let result = server
            .execute_tool("search_documents", params, &ctx)
            .unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_builtin_add_document() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let ctx = ToolContext::new();
        let params = serde_json::json!({
            "id": "doc1",
            "content": "Some document content",
            "metadata": {"source": "test"},
        });
        let result = server.execute_tool("add_document", params, &ctx).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_builtin_search_documents_missing_query() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let ctx = ToolContext::new();
        let result = server.execute_tool("search_documents", serde_json::json!({}), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_add_document_missing_id() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let ctx = ToolContext::new();
        let result = server.execute_tool("add_document", serde_json::json!({}), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_add_document_missing_content() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let ctx = ToolContext::new();
        let result = server.execute_tool("add_document", serde_json::json!({"id": "x"}), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_chunk_text_missing_text() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let ctx = ToolContext::new();
        let result = server.execute_tool("chunk_text", serde_json::json!({}), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_execute_rule_missing_rule_id() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();
        let ctx = ToolContext::new();
        let result = server.execute_tool("execute_rule", serde_json::json!({"context": {}}), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_context_default_impl() {
        let ctx = ToolContext::default();
        assert!(ctx.document_store.is_none());
        assert!(ctx.knowledge_base.is_none());
        assert!(ctx.data.is_empty());
    }

    #[test]
    fn test_mcp_server_default_impl() {
        let server = MCPServer::default();
        assert!(server.list_tools().is_empty());
    }
}
