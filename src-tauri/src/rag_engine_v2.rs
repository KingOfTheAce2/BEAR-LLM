use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;
use std::path::PathBuf;
use crate::embeddings::EmbeddingsEngine;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: JsonValue,
    pub embeddings: Vec<f32>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub content: String,
    pub score: f32,
    pub metadata: JsonValue,
    pub reasoning: Option<String>,
}

pub struct RAGEngineV2 {
    documents: Arc<RwLock<HashMap<String, Document>>>,
    index_path: PathBuf,
    embeddings_engine: Arc<RwLock<EmbeddingsEngine>>,
    chunk_size: usize,
    chunk_overlap: usize,
    reranking_enabled: bool,
}

impl RAGEngineV2 {
    pub fn new() -> Self {
        let index_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("bear-ai-llm")
            .join("rag_index");

        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            index_path,
            embeddings_engine: Arc::new(RwLock::new(EmbeddingsEngine::new())),
            chunk_size: 512,
            chunk_overlap: 50,
            reranking_enabled: true,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Create index directory
        tokio::fs::create_dir_all(&self.index_path).await?;

        // Initialize embeddings engine
        let mut engine = self.embeddings_engine.write().await;
        engine.initialize().await?;
        drop(engine);

        // Load existing index
        self.load_index().await?;

        Ok(())
    }

    pub async fn add_document(&self, content: &str, metadata: JsonValue) -> Result<String> {
        let doc_id = Uuid::new_v4().to_string();
        let chunks = self.chunk_text(content);
        let mut documents_map = self.documents.write().await;

        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_id = format!("{}_{}", doc_id, i);

            // Generate real embeddings
            let engine = self.embeddings_engine.read().await;
            let embeddings = engine.generate_single_embedding(chunk).await?;
            drop(engine);

            let document = Document {
                id: chunk_id.clone(),
                content: chunk.clone(),
                metadata: metadata.clone(),
                embeddings,
                timestamp: chrono::Utc::now().timestamp(),
            };

            documents_map.insert(chunk_id, document);
        }

        // Save index after adding documents
        drop(documents_map);
        self.save_index().await?;

        Ok(doc_id)
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Generate query embeddings
        let engine = self.embeddings_engine.read().await;
        let query_embedding = engine.generate_single_embedding(query).await?;
        drop(engine);

        let documents = self.documents.read().await;
        let mut results: Vec<(String, f32, Document)> = Vec::new();

        // Calculate similarity scores
        for (id, doc) in documents.iter() {
            let score = EmbeddingsEngine::cosine_similarity(&query_embedding, &doc.embeddings);
            if score > 0.3 { // Threshold for relevance
                results.push((id.clone(), score, doc.clone()));
            }
        }

        // Sort by score
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Apply reranking if enabled
        if self.reranking_enabled && !results.is_empty() {
            results = self.rerank_results(query, results).await?;
        }

        // Limit results
        results.truncate(limit);

        // Convert to SearchResult
        let search_results: Vec<SearchResult> = results
            .into_iter()
            .map(|(id, score, doc)| SearchResult {
                document_id: id,
                content: doc.content,
                score,
                metadata: doc.metadata,
                reasoning: None,
            })
            .collect();

        Ok(search_results)
    }

    pub async fn agentic_search(&self, query: &str, context: &str) -> Result<Vec<SearchResult>> {
        // Enhanced query with context
        let enhanced_query = self.enhance_query(query, context).await?;

        // Perform search with enhanced query
        let mut results = self.search(&enhanced_query, 10).await?;

        // Generate reasoning for each result
        for result in &mut results {
            result.reasoning = Some(self.generate_reasoning(query, &result.content).await?);
        }

        Ok(results)
    }

    async fn enhance_query(&self, query: &str, context: &str) -> Result<String> {
        // Query enhancement logic
        // This could use an LLM for query rewriting, but for now we'll use a simpler approach

        let keywords = self.extract_keywords(query);
        let context_keywords = self.extract_keywords(context);

        let mut enhanced = query.to_string();
        for keyword in context_keywords.iter().take(3) {
            if !keywords.contains(keyword) {
                enhanced.push_str(&format!(" {}", keyword));
            }
        }

        Ok(enhanced)
    }

    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // Simple keyword extraction
        let stop_words = vec!["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for"];

        text.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 3 && !stop_words.contains(word))
            .map(|s| s.to_string())
            .collect()
    }

    async fn rerank_results(
        &self,
        query: &str,
        mut results: Vec<(String, f32, Document)>,
    ) -> Result<Vec<(String, f32, Document)>> {
        // Cross-encoder reranking simulation
        // In production, this would use a cross-encoder model

        for result in &mut results {
            // Boost score based on keyword matching
            let query_lower = query.to_lowercase();
            let content_lower = result.2.content.to_lowercase();

            let mut boost = 0.0;
            for word in query_lower.split_whitespace() {
                if content_lower.contains(word) {
                    boost += 0.05;
                }
            }

            // Check for exact phrase match
            if content_lower.contains(&query_lower) {
                boost += 0.2;
            }

            result.1 += boost;
        }

        // Re-sort after reranking
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(results)
    }

    async fn generate_reasoning(&self, query: &str, content: &str) -> Result<String> {
        // Generate reasoning for why this document is relevant
        // In production, this would use an LLM

        let query_words: Vec<&str> = query.split_whitespace().collect();
        let mut matching_words = Vec::new();

        for word in &query_words {
            if content.to_lowercase().contains(&word.to_lowercase()) {
                matching_words.push(*word);
            }
        }

        if matching_words.is_empty() {
            Ok("Semantically similar content based on context.".to_string())
        } else {
            Ok(format!(
                "Document contains relevant terms: {}. Content aligns with the query context.",
                matching_words.join(", ")
            ))
        }
    }

    fn chunk_text(&self, text: &str) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();
        let mut i = 0;

        while i < words.len() {
            let end = std::cmp::min(i + self.chunk_size, words.len());
            let chunk = words[i..end].join(" ");
            chunks.push(chunk);

            // Move forward with overlap
            i += self.chunk_size - self.chunk_overlap;
        }

        chunks
    }

    async fn save_index(&self) -> Result<()> {
        let documents = self.documents.read().await;
        let index_file = self.index_path.join("index.json");

        let serialized = serde_json::to_string(&*documents)?;
        tokio::fs::write(index_file, serialized).await?;

        Ok(())
    }

    async fn load_index(&self) -> Result<()> {
        let index_file = self.index_path.join("index.json");

        if index_file.exists() {
            let data = tokio::fs::read_to_string(index_file).await?;
            let loaded_docs: HashMap<String, Document> = serde_json::from_str(&data)?;

            let mut documents = self.documents.write().await;
            *documents = loaded_docs;
        }

        Ok(())
    }

    pub async fn delete_document(&self, doc_id: &str) -> Result<()> {
        let mut documents = self.documents.write().await;

        // Remove all chunks for this document
        let keys_to_remove: Vec<String> = documents
            .keys()
            .filter(|k| k.starts_with(doc_id))
            .cloned()
            .collect();

        for key in keys_to_remove {
            documents.remove(&key);
        }

        drop(documents);
        self.save_index().await?;

        Ok(())
    }

    pub async fn clear_index(&self) -> Result<()> {
        let mut documents = self.documents.write().await;
        documents.clear();

        drop(documents);
        self.save_index().await?;

        Ok(())
    }

    pub async fn get_statistics(&self) -> Result<JsonValue> {
        let documents = self.documents.read().await;

        let stats = serde_json::json!({
            "total_chunks": documents.len(),
            "index_path": self.index_path.to_string_lossy(),
            "chunk_size": self.chunk_size,
            "chunk_overlap": self.chunk_overlap,
            "reranking_enabled": self.reranking_enabled,
        });

        Ok(stats)
    }
}