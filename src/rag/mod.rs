//! RAG (Retrieval-Augmented Generation) and Rule-Based System Support
//!
//! Provides infrastructure for:
//! - Document storage and retrieval for RAG systems
//! - Vector embeddings management
//! - Knowledge base operations
//! - Rule-based system integration
//! - Cache management for retrieval
//! - Database connectivity abstractions
//! - Persistent storage with SQLite
//! - High-performance KV store with Sled
//! - Vector database integration

pub mod cache;
pub mod database;
pub mod documents;
pub mod knowledge;
pub mod mcp;
pub mod rules;
pub mod vector;

// Re-export all public types for backward compatibility
pub use cache::{CacheStats, RetrievalCache};
pub use database::{Database, InMemoryDatabase};
pub use documents::{ChunkInfo, Document, DocumentStore};
pub use knowledge::{KnowledgeBase, KnowledgeBaseConfig};
pub use mcp::{MCPServer, MCPTool, ToolContext, ToolExecutor, ToolResult};
pub use rules::{Rule, RuleAction, RuleCondition, RuleEngine};
pub use vector::{cosine_similarity, SimpleVectorStore, VectorStore};

#[cfg(feature = "sqlite")]
pub use database::SQLiteDatabase;

#[cfg(feature = "kv-store")]
pub use database::SledDatabase;

#[cfg(feature = "vector-db")]
pub use vector::QdrantVectorStore;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_document_store() {
        let mut store = DocumentStore::new();

        let doc = Document {
            id: "doc1".to_string(),
            content: "Test content".to_string(),
            metadata: HashMap::new(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
            chunk_info: None,
        };

        store.add_document(doc.clone()).unwrap();
        assert_eq!(store.count(), 1);

        let retrieved = store.get_document("doc1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test content");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_knowledge_base_chunking() {
        let config = KnowledgeBaseConfig {
            chunk_size: 10,
            chunk_overlap: 2,
            ..Default::default()
        };

        let kb = KnowledgeBase::new("test".to_string(), config);
        let text = "This is a test document with multiple chunks";
        let chunks = kb.chunk_text(text, "doc1");

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].chunk_info.as_ref().unwrap().chunk_index, 0);
    }

    #[test]
    fn test_rule_engine() {
        let mut engine = RuleEngine::new();

        let rule = Rule {
            id: "rule1".to_string(),
            name: "Test Rule".to_string(),
            conditions: {
                let mut cond = HashMap::new();
                cond.insert(
                    "status".to_string(),
                    RuleCondition::Equals("active".to_string()),
                );
                cond
            },
            actions: vec![RuleAction::SetValue {
                key: "processed".to_string(),
                value: "true".to_string(),
            }],
            priority: 10,
            enabled: true,
        };

        engine.add_rule(rule);
        engine.set_context("status".to_string(), "active".to_string());

        let executed = engine.execute().unwrap();
        assert_eq!(executed.len(), 1);
        assert_eq!(engine.get_context("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_retrieval_cache() {
        let mut cache = RetrievalCache::new(1024);

        let doc = Document {
            id: "doc1".to_string(),
            content: "Test".to_string(),
            metadata: HashMap::new(),
            embedding: None,
            chunk_info: None,
        };

        cache.cache_results("query1", vec![doc.clone()]).unwrap();

        let cached = cache.get_cached("query1");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap()[0].id, "doc1");
    }

    #[test]
    fn test_in_memory_database() {
        let mut db = InMemoryDatabase::new();
        db.create_table("users".to_string());

        let mut data = HashMap::new();
        data.insert("id".to_string(), "1".to_string());
        data.insert("name".to_string(), "Alice".to_string());

        db.insert("users", data).unwrap();

        let results = db.query("users WHERE id=1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("name"), Some(&"Alice".to_string()));
    }

    #[test]
    fn test_mcp_tool_creation() {
        let tool = MCPTool::new("test_tool".to_string(), "A test tool".to_string())
            .add_parameter("param1", "string", "First parameter", true)
            .add_parameter("param2", "number", "Second parameter", false);

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");

        let props = tool.input_schema.get("properties").unwrap();
        assert!(props.get("param1").is_some());
        assert!(props.get("param2").is_some());
    }

    #[test]
    fn test_tool_context() {
        let ctx = ToolContext::new()
            .with_document_store("store1".to_string())
            .with_knowledge_base("kb1".to_string())
            .with_data("key1".to_string(), "value1".to_string());

        assert_eq!(ctx.document_store, Some("store1".to_string()));
        assert_eq!(ctx.knowledge_base, Some("kb1".to_string()));
        assert_eq!(ctx.data.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_tool_result() {
        let success = ToolResult::success(serde_json::json!({"result": "ok"}));
        assert!(success.success);
        assert!(success.error.is_none());

        let failure = ToolResult::failure("Error occurred".to_string());
        assert!(!failure.success);
        assert_eq!(failure.error, Some("Error occurred".to_string()));
    }

    #[test]
    fn test_mcp_server_registration() {
        let mut server = MCPServer::new();

        let tool = MCPTool::new("simple_tool".to_string(), "A simple tool".to_string());

        server
            .register_tool(tool, |_params, _ctx| {
                Ok(ToolResult::success(serde_json::json!({"status": "ok"})))
            })
            .unwrap();

        let tools = server.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "simple_tool");
    }

    #[test]
    fn test_mcp_server_execution() {
        let mut server = MCPServer::new();

        let tool = MCPTool::new("echo_tool".to_string(), "Echoes the input".to_string());

        server
            .register_tool(tool, |params, _ctx| Ok(ToolResult::success(params)))
            .unwrap();

        let ctx = ToolContext::new();
        let params = serde_json::json!({"message": "hello"});
        let result = server
            .execute_tool("echo_tool", params.clone(), &ctx)
            .unwrap();

        assert!(result.success);
        assert_eq!(result.data, params);
    }

    #[test]
    fn test_mcp_builtin_tools() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();

        let tools = server.list_tools();
        assert!(tools.len() >= 4); // At least 4 built-in tools

        // Check for specific tools
        assert!(server.get_tool("search_documents").is_some());
        assert!(server.get_tool("add_document").is_some());
        assert!(server.get_tool("chunk_text").is_some());
        assert!(server.get_tool("execute_rule").is_some());
    }

    #[test]
    fn test_builtin_search_tool() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();

        let ctx = ToolContext::new();
        let params = serde_json::json!({
            "query": "test query",
            "top_k": 3
        });

        let result = server
            .execute_tool("search_documents", params, &ctx)
            .unwrap();
        assert!(result.success);
        assert_eq!(
            result.data.get("query").unwrap().as_str().unwrap(),
            "test query"
        );
        assert_eq!(result.data.get("top_k").unwrap().as_u64().unwrap(), 3);
    }

    #[test]
    fn test_builtin_chunk_tool() {
        let mut server = MCPServer::new();
        server.register_builtin_tools().unwrap();

        let ctx = ToolContext::new();
        let params = serde_json::json!({
            "text": "This is a test text that will be chunked",
            "chunk_size": 10,
            "overlap": 2
        });

        let result = server.execute_tool("chunk_text", params, &ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.data.get("chunk_size").unwrap().as_u64().unwrap(), 10);
        assert_eq!(result.data.get("overlap").unwrap().as_u64().unwrap(), 2);
        // Verify actual chunks were produced
        let chunks = result.data.get("chunks").unwrap().as_array().unwrap();
        assert!(!chunks.is_empty());
        assert_eq!(
            result.data.get("num_chunks").unwrap().as_u64().unwrap(),
            chunks.len() as u64
        );
    }
}
