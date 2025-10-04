use crate::utils::cosine_similarity;
use anyhow::{anyhow, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Production RAG Engine with real embeddings and vector search
/// Uses FastEmbed as the embedding backend

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub embeddings: Vec<f32>,
    pub metadata: JsonValue,
    pub timestamp: i64,
    pub chunk_index: usize,
    pub total_chunks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub content: String,
    pub score: f32,
    pub metadata: JsonValue,
    pub highlight: Option<String>,
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub embedding_model: String,
    pub similarity_threshold: f32,
    pub max_results: usize,
    pub enable_reranking: bool,
    pub enable_hybrid_search: bool,
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 50,
            embedding_model: "BAAI/bge-small-en-v1.5".to_string(),
            similarity_threshold: 0.3,
            max_results: 10,
            enable_reranking: true,
            enable_hybrid_search: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGModelInfo {
    pub name: String,
    pub model_id: String,
    pub description: String,
    pub dimensions: usize,
    pub size_mb: u64,
    pub use_case: String,
    pub is_active: bool,
}

pub struct RAGEngine {
    documents: Arc<RwLock<HashMap<String, Document>>>,
    embeddings_model: Arc<RwLock<Option<TextEmbedding>>>,
    config: Arc<RwLock<RAGConfig>>,
    index_path: PathBuf,
    inverted_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl Default for RAGEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RAGEngine {
    pub fn new() -> Self {
        let index_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("bear-ai-llm")
            .join("rag_index");

        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            embeddings_model: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(RAGConfig::default())),
            index_path,
            inverted_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_available_models() -> Vec<RAGModelInfo> {
        vec![
            RAGModelInfo {
                name: "BGE Small EN v1.5".to_string(),
                model_id: "BAAI/bge-small-en-v1.5".to_string(),
                description: "Fast, efficient, general-purpose embeddings".to_string(),
                dimensions: 384,
                size_mb: 150,
                use_case: "General documents, fast search".to_string(),
                is_active: true,
            },
            RAGModelInfo {
                name: "BGE Base EN v1.5".to_string(),
                model_id: "BAAI/bge-base-en-v1.5".to_string(),
                description: "Balanced quality and speed".to_string(),
                dimensions: 768,
                size_mb: 440,
                use_case: "Better accuracy, moderate speed".to_string(),
                is_active: false,
            },
            RAGModelInfo {
                name: "BGE Large EN v1.5".to_string(),
                model_id: "BAAI/bge-large-en-v1.5".to_string(),
                description: "Highest quality embeddings".to_string(),
                dimensions: 1024,
                size_mb: 1340,
                use_case: "Best accuracy, requires more resources".to_string(),
                is_active: false,
            },
            RAGModelInfo {
                name: "All MiniLM L6 v2".to_string(),
                model_id: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
                description: "Lightweight, very fast".to_string(),
                dimensions: 384,
                size_mb: 90,
                use_case: "Resource-constrained systems".to_string(),
                is_active: false,
            },
        ]
    }

    pub async fn initialize(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.index_path).await?;
        self.load_index().await?;
        tracing::info!(
            "✅ RAG Engine initialized with {} chunks",
            self.documents.read().await.len()
        );
        Ok(())
    }

    async fn ensure_embeddings_model(&self) -> Result<()> {
    if self.embeddings_model.read().await.is_some() {
        return Ok(());
    }

    let model_name = self.config.read().await.embedding_model.clone();

    // Fix applied here: Convert the String error from try_from into an anyhow::Error.
    let embedding_model = EmbeddingModel::try_from(model_name.clone())
        .map_err(|e| anyhow!("Failed to create EmbeddingModel from name '{}': {}", model_name, e))?;

    // Initialize embedding model directly
    let model = TextEmbedding::try_new(
        InitOptions::new(embedding_model)
            .with_show_download_progress(true),
    )?;

    let mut lock = self.embeddings_model.write().await;
    *lock = Some(model);

    tracing::info!("✅ Loaded embedding model: {}", model_name);
    Ok(())
}
    pub fn is_initialized(&self) -> bool {
        true
    }

    pub async fn switch_rag_model(&self, model_id: String) -> Result<()> {
        self.config.write().await.embedding_model = model_id.clone();
        *self.embeddings_model.write().await = None;
        tracing::info!("🔄 RAG model switched to: {}", model_id);
        Ok(())
    }

    pub async fn get_active_model(&self) -> String {
        self.config.read().await.embedding_model.clone()
    }

    pub async fn get_config(&self) -> RAGConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, new_config: RAGConfig) -> Result<()> {
        *self.config.write().await = new_config;
        *self.embeddings_model.write().await = None;
        Ok(())
    }

    pub async fn add_document(&self, content: &str, metadata: JsonValue) -> Result<String> {
        self.ensure_embeddings_model().await?;
        let doc_id = Uuid::new_v4().to_string();

        let chunks = self.chunk_text(content).await;
        let total_chunks = chunks.len();

        let mut model_lock = self.embeddings_model.write().await;
        let model = model_lock.as_mut().ok_or_else(|| anyhow!("Model not initialized"))?;

        let mut documents = self.documents.write().await;
        let mut inverted_index = self.inverted_index.write().await;

        for (idx, chunk) in chunks.iter().enumerate() {
            let embeddings = model.embed(vec![chunk.as_str()], None)?
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("Failed to embed text"))?;

            let chunk_id = format!("{}_{}", doc_id, idx);
            documents.insert(
                chunk_id.clone(),
                Document {
                    id: chunk_id.clone(),
                    content: chunk.clone(),
                    embeddings,
                    metadata: metadata.clone(),
                    timestamp: chrono::Utc::now().timestamp(),
                    chunk_index: idx,
                    total_chunks,
                },
            );
            self.update_inverted_index(&chunk_id, chunk, &mut inverted_index);
        }

        self.save_index().await?;
        Ok(doc_id)
    }

    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        self.ensure_embeddings_model().await?;

        let config = self.config.read().await.clone();
        let limit = limit.unwrap_or(config.max_results);

        let mut model_lock = self.embeddings_model.write().await;
        let model = model_lock.as_mut().ok_or_else(|| anyhow!("Model not initialized"))?;

        let query_embedding = model
            .embed(vec![query], None)?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Failed to embed query"))?;

        let mut results = if config.enable_hybrid_search {
            self.hybrid_search(query, &query_embedding, limit).await?
        } else {
            self.vector_search(&query_embedding, limit).await?
        };

        if config.enable_reranking && !results.is_empty() {
            results = self.rerank_results(query, results).await?;
        }

        Ok(results)
    }

    async fn vector_search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        let documents = self.documents.read().await;
        let config = self.config.read().await;
        let mut scores: Vec<(String, f32, Document)> = Vec::new();

        for (id, doc) in documents.iter() {
            let similarity = cosine_similarity(query_embedding, &doc.embeddings);
            if similarity >= config.similarity_threshold {
                scores.push((id.clone(), similarity, doc.clone()));
            }
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(limit);

        Ok(scores
            .into_iter()
            .map(|(id, score, doc)| SearchResult {
                document_id: id,
                content: doc.content,
                score,
                metadata: doc.metadata,
                highlight: None,
                reasoning: None,
            })
            .collect())
    }

    async fn hybrid_search(
        &self,
        query: &str,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let vector_results = self.vector_search(query_embedding, limit * 2).await?;
        let keyword_results = self.keyword_search(query, limit * 2).await?;

        let mut merged: HashMap<String, (f32, SearchResult)> = HashMap::new();
        for result in vector_results {
            merged.insert(result.document_id.clone(), (result.score * 0.7, result));
        }
        for result in keyword_results {
            merged
                .entry(result.document_id.clone())
                .and_modify(|e| e.0 += result.score * 0.3)
                .or_insert((result.score * 0.3, result));
        }

        let mut results: Vec<SearchResult> = merged
            .into_iter()
            .map(|(_, (score, mut r))| {
                r.score = score;
                r
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    async fn keyword_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let docs = self.documents.read().await;
        let index = self.inverted_index.read().await;

        let mut scores = HashMap::new();
        let tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|t| t.len() > 2)
            .map(|s| s.to_string())
            .collect();

        for token in &tokens {
            if let Some(ids) = index.get(token) {
                for id in ids {
                    *scores.entry(id.clone()).or_insert(0.0) += 1.0;
                }
            }
        }

        let max_score = scores.values().cloned().fold(0.0, f32::max);
        let mut results = Vec::new();
        if max_score > 0.0 {
            for (id, score) in scores {
                if let Some(doc) = docs.get(&id) {
                    results.push(SearchResult {
                        document_id: id,
                        content: doc.content.clone(),
                        score: score / max_score,
                        metadata: doc.metadata.clone(),
                        highlight: self.generate_highlight(&doc.content, &tokens),
                        reasoning: None,
                    });
                }
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    async fn rerank_results(&self, query: &str, mut results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        let query_lower = query.to_lowercase();
        let tokens: Vec<&str> = query_lower.split_whitespace().collect();

        for r in &mut results {
            let text = r.content.to_lowercase();
            let mut boost = 0.0;
            if text.contains(&query_lower) {
                boost += 0.3;
            }
            if tokens.iter().all(|t| text.contains(t)) {
                boost += 0.2;
            }
            if tokens.len() > 1 {
                let p = self.calculate_token_proximity(&text, &tokens);
                boost += p * 0.1;
            }
            r.score = (r.score + boost).min(1.0);
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    fn calculate_token_proximity(&self, text: &str, tokens: &[&str]) -> f32 {
        if tokens.len() < 2 {
            return 0.0;
        }
        let mut positions = Vec::new();
        for token in tokens {
            let mut pos = Vec::new();
            let mut start = 0;
            while let Some(p) = text[start..].find(token) {
                pos.push(start + p);
                start += p + token.len();
            }
            if pos.is_empty() {
                return 0.0;
            }
            positions.push(pos);
        }
        let mut min_span = text.len();
        for f in &positions[0] {
            let mut max_p = *f;
            let mut min_p = *f;
            for tpos in &positions[1..] {
                if let Some(&closest) = tpos.iter().min_by_key(|&&p| (p as i32 - *f as i32).abs()) {
                    max_p = max_p.max(closest);
                    min_p = min_p.min(closest);
                }
            }
            min_span = min_span.min(max_p - min_p);
        }
        let max_dist = 100;
        1.0 - (min_span.min(max_dist) as f32 / max_dist as f32)
    }

    fn generate_highlight(&self, content: &str, tokens: &[String]) -> Option<String> {
        let text = content.to_lowercase();
        let mut best_start = 0;
        let mut best_score = 0;
        let window = 150;
        for i in 0..text.len().saturating_sub(window) {
            let end = std::cmp::min(i + window, text.len());
            let slice = &text[i..end];
            let score = tokens.iter().filter(|t| slice.contains(t.as_str())).count();
            if score > best_score {
                best_score = score;
                best_start = i;
            }
        }
        if best_score > 0 {
            let end = std::cmp::min(best_start + window, text.len());
            let mut highlight = content[best_start..end].to_string();
            if best_start > 0 {
                highlight = format!("...{}", highlight);
            }
            if end < content.len() {
                highlight = format!("{}...", highlight);
            }
            Some(highlight)
        } else {
            None
        }
    }

    async fn chunk_text(&self, text: &str) -> Vec<String> {
        let cfg = self.config.read().await;
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();
        if words.is_empty() {
            return chunks;
        }
        let mut i = 0;
        while i < words.len() {
            let end = std::cmp::min(i + cfg.chunk_size, words.len());
            chunks.push(words[i..end].join(" "));
            if end < words.len() {
                i += cfg.chunk_size - cfg.chunk_overlap;
            } else {
                break;
            }
        }
        chunks
    }

    fn update_inverted_index(
        &self,
        doc_id: &str,
        text: &str,
        index: &mut HashMap<String, Vec<String>>,
    ) {
        for token in text
            .to_lowercase()
            .split_whitespace()
            .filter(|t| t.len() > 2)
        {
            index.entry(token.to_string()).or_default().push(doc_id.to_string());
        }
    }

    async fn save_index(&self) -> Result<()> {
        let docs = self.documents.read().await;
        let index = self.inverted_index.read().await;
        tokio::fs::create_dir_all(&self.index_path).await?;
        tokio::fs::write(
            self.index_path.join("documents.json"),
            serde_json::to_string(&*docs            )?,
        )
        .await?;

        tokio::fs::write(
            self.index_path.join("inverted_index.json"),
            serde_json::to_string(&*index)?,
        )
        .await?;

        Ok(())
    }

    async fn load_index(&self) -> Result<()> {
        let index_file = self.index_path.join("documents.json");
        if index_file.exists() {
            let data = tokio::fs::read_to_string(&index_file).await?;
            let loaded_docs: HashMap<String, Document> = serde_json::from_str(&data)?;
            *self.documents.write().await = loaded_docs;
        }

        let inverted_file = self.index_path.join("inverted_index.json");
        if inverted_file.exists() {
            let data = tokio::fs::read_to_string(&inverted_file).await?;
            let loaded_index: HashMap<String, Vec<String>> = serde_json::from_str(&data)?;
            *self.inverted_index.write().await = loaded_index;
        }

        Ok(())
    }

    pub async fn clear_cache(&self) -> Result<()> {
        *self.embeddings_model.write().await = None;
        tracing::info!("🧹 RAG engine embedding cache cleared");
        Ok(())
    }

    pub async fn clear_index(&self) -> Result<()> {
        self.documents.write().await.clear();
        self.inverted_index.write().await.clear();
        self.save_index().await?;
        tracing::info!("🧹 RAG document index cleared");
        Ok(())
    }

    pub async fn delete_document(&self, doc_id: &str) -> Result<()> {
        let mut docs = self.documents.write().await;
        let mut index = self.inverted_index.write().await;

        let keys_to_remove: Vec<String> = docs
            .keys()
            .filter(|k| k.starts_with(doc_id))
            .cloned()
            .collect();

        for key in &keys_to_remove {
            if let Some(doc) = docs.get(key) {
                for token in doc
                    .content
                    .to_lowercase()
                    .split_whitespace()
                    .filter(|t| t.len() > 2)
                {
                    if let Some(ids) = index.get_mut(token) {
                        ids.retain(|id| id != key);
                    }
                }
            }
            docs.remove(key);
        }

        drop(docs);
        drop(index);
        self.save_index().await?;
        tracing::info!("🗑️ Document {} deleted from index", doc_id);
        Ok(())
    }

    pub async fn get_statistics(&self) -> Result<JsonValue> {
        let docs = self.documents.read().await;
        let index = self.inverted_index.read().await;
        let cfg = self.config.read().await;

        let mut unique_docs = HashSet::new();
        for k in docs.keys() {
            if let Some(base) = k.split('_').next() {
                unique_docs.insert(base);
            }
        }

        Ok(serde_json::json!({
            "total_chunks": docs.len(),
            "unique_documents": unique_docs.len(),
            "vocab_size": index.len(),
            "index_bytes": self.estimate_index_size(&docs, &index),
            "config": {
                "chunk_size": cfg.chunk_size,
                "chunk_overlap": cfg.chunk_overlap,
                "embedding_model": cfg.embedding_model,
                "enable_hybrid_search": cfg.enable_hybrid_search,
                "enable_reranking": cfg.enable_reranking
            }
        }))
    }

    fn estimate_index_size(
        &self,
        docs: &HashMap<String, Document>,
        index: &HashMap<String, Vec<String>>,
    ) -> usize {
        let doc_bytes: usize = docs
            .iter()
            .map(|(id, d)| id.len() + d.content.len() + d.embeddings.len() * 4)
            .sum();
        let index_bytes: usize = index
            .iter()
            .map(|(k, v)| k.len() + v.iter().map(|x| x.len()).sum::<usize>())
            .sum();
        doc_bytes + index_bytes
    }

    /// Build final prompt for LLM with retrieved context
    pub async fn generate_augmented_prompt(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<String> {
        let results = self.search(query, limit).await?;

        if results.is_empty() {
            return Ok(format!(
                "INSTRUCTION: Answer the following question to the best of your ability.\n\nQUESTION: {}\nANSWER:",
                query
            ));
        }

        let mut context = String::new();
        for (i, r) in results.iter().enumerate() {
            let header = format!("--- SOURCE {} (Relevance: {:.2}) ---\n", i + 1, r.score);
            let content = r.highlight.clone().unwrap_or_else(|| r.content.clone());
            let meta = if let Some(obj) = r.metadata.as_object() {
                let m: Vec<String> = obj
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|val| format!("{}: {}", k, val)))
                    .collect();
                if !m.is_empty() {
                    format!("Metadata: {}\n", m.join(", "))
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            context.push_str(&format!("{}{}\n{}\n\n", header, meta, content));
        }

        let prompt = format!(
            "SYSTEM INSTRUCTION: You are a legal AI assistant. Use ONLY the provided context. \
             If context is insufficient, say so clearly.\n\nCONTEXT:\n{}\nQUESTION: {}\n\nANSWER:",
            context, query
        );

        tracing::info!(
            "🧠 Generated augmented prompt with {} sources ({} chars)",
            results.len(),
            prompt.len()
        );

        Ok(prompt)
    }
}