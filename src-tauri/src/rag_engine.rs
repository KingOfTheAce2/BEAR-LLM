use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use uuid::Uuid;
use crate::utils::cosine_similarity;

// Production RAG Engine with real embeddings and vector search
// This is the single source of truth for RAG functionality in BEAR AI

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
    #[serde(skip)]
    #[allow(dead_code)]
    pub embedding_model: EmbeddingModel,
    pub description: String,
    pub dimensions: usize,
    pub size_mb: u64,
    pub use_case: String,
    pub is_active: bool,
}

impl RAGEngine {
    pub fn get_available_models() -> Vec<RAGModelInfo> {
        vec![
            RAGModelInfo {
                name: "BGE Small EN v1.5".to_string(),
                model_id: "BAAI/bge-small-en-v1.5".to_string(),
                embedding_model: EmbeddingModel::BGESmallENV15,
                description: "Fast, efficient, general-purpose embeddings".to_string(),
                dimensions: 384,
                size_mb: 150,
                use_case: "General documents, fast search".to_string(),
                is_active: true,
            },
            RAGModelInfo {
                name: "BGE Base EN v1.5".to_string(),
                model_id: "BAAI/bge-base-en-v1.5".to_string(),
                embedding_model: EmbeddingModel::BGEBaseENV15,
                description: "Balanced quality and speed".to_string(),
                dimensions: 768,
                size_mb: 440,
                use_case: "Better accuracy, moderate speed".to_string(),
                is_active: false,
            },
            RAGModelInfo {
                name: "BGE Large EN v1.5".to_string(),
                model_id: "BAAI/bge-large-en-v1.5".to_string(),
                embedding_model: EmbeddingModel::BGELargeENV15,
                description: "Highest quality embeddings".to_string(),
                dimensions: 1024,
                size_mb: 1340,
                use_case: "Best accuracy, requires more resources".to_string(),
                is_active: false,
            },
            RAGModelInfo {
                name: "All MiniLM L6 v2".to_string(),
                model_id: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
                embedding_model: EmbeddingModel::AllMiniLML6V2,
                description: "Lightweight, very fast".to_string(),
                dimensions: 384,
                size_mb: 90,
                use_case: "Resource-constrained systems".to_string(),
                is_active: false,
            },
        ]
    }
}

pub struct RAGEngine {
    documents: Arc<RwLock<HashMap<String, Document>>>,
    embeddings_model: Arc<RwLock<Option<TextEmbedding>>>,
    config: Arc<RwLock<RAGConfig>>,
    index_path: PathBuf,
    inverted_index: Arc<RwLock<HashMap<String, Vec<String>>>>, // For keyword search
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

    pub async fn initialize(&self) -> Result<()> {
        // Create index directory only
        tokio::fs::create_dir_all(&self.index_path).await?;

        // Load existing index (fast)
        self.load_index().await?;

        tracing::info!("✅ RAG Engine initialized with {} documents (embeddings will load on first use)",
                 self.documents.read().await.len());

        Ok(())
    }

    // Lazy-load embeddings model on first use (if not downloaded during setup)
    async fn ensure_embeddings_model(&self) -> Result<()> {
        let model_lock = self.embeddings_model.read().await;
        if model_lock.is_some() {
            return Ok(());
        }
        drop(model_lock);

        // Get configured model from config
        let config = self.config.read().await;
        let model_id = config.embedding_model.clone();
        drop(config);

        // Map model ID to EmbeddingModel enum
        let embedding_model = self.get_embedding_model_from_id(&model_id)?;

        // If model was already downloaded during setup, this will be instant
        // If not, it will download now (fallback)
        tracing::info!("📥 Loading RAG embeddings model: {}...", model_id);

        // Initialize embeddings model (uses cache if available)
        let model = TextEmbedding::try_new(
            InitOptions::new(embedding_model)
                .with_show_download_progress(true)
        )?;

        let mut model_lock = self.embeddings_model.write().await;
        *model_lock = Some(model);

        tracing::info!("✅ RAG embeddings model ready: {}", model_id);
        Ok(())
    }

    fn get_embedding_model_from_id(&self, model_id: &str) -> Result<EmbeddingModel> {
        match model_id {
            "BAAI/bge-small-en-v1.5" => Ok(EmbeddingModel::BGESmallENV15),
            "BAAI/bge-base-en-v1.5" => Ok(EmbeddingModel::BGEBaseENV15),
            "BAAI/bge-large-en-v1.5" => Ok(EmbeddingModel::BGELargeENV15),
            "sentence-transformers/all-MiniLM-L6-v2" => Ok(EmbeddingModel::AllMiniLML6V2),
            _ => Err(anyhow!("Unsupported embedding model: {}", model_id)),
        }
    }

    pub async fn switch_rag_model(&self, model_id: String) -> Result<()> {
        tracing::info!("🔄 Switching RAG model to: {}", model_id);

        // Validate model ID
        self.get_embedding_model_from_id(&model_id)?;

        // Update config
        let mut config = self.config.write().await;
        config.embedding_model = model_id.clone();
        drop(config);

        // Unload current model
        let mut model_lock = self.embeddings_model.write().await;
        *model_lock = None;
        drop(model_lock);

        tracing::info!("✅ RAG model switched to: {}. Will load on next use.", model_id);

        Ok(())
    }

    pub async fn get_active_model(&self) -> String {
        let config = self.config.read().await;
        config.embedding_model.clone()
    }

    pub async fn get_config(&self) -> RAGConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, new_config: RAGConfig) -> Result<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }

    pub async fn add_document(&self, content: &str, metadata: JsonValue) -> Result<String> {
        // SECURITY: Prevent memory exhaustion from huge documents
        const MAX_DOCUMENT_SIZE: usize = 100_000_000; // 100MB
        const MAX_CHUNKS_PER_DOCUMENT: usize = 10_000;

        if content.len() > MAX_DOCUMENT_SIZE {
            return Err(anyhow!(
                "Document exceeds maximum size of {}MB (got {}MB)",
                MAX_DOCUMENT_SIZE / 1_000_000,
                content.len() / 1_000_000
            ));
        }

        // Ensure embeddings model is loaded (lazy load on first use)
        self.ensure_embeddings_model().await?;

        let doc_id = Uuid::new_v4().to_string();
        let chunks = self.chunk_text(content).await;
        let total_chunks = chunks.len();

        if total_chunks > MAX_CHUNKS_PER_DOCUMENT {
            return Err(anyhow!(
                "Document produces too many chunks: {} (max: {}). \
                Consider splitting the document or increasing chunk_size.",
                total_chunks,
                MAX_CHUNKS_PER_DOCUMENT
            ));
        }

        let mut model_lock = self.embeddings_model.write().await;
        let model = model_lock.as_mut()
            .ok_or_else(|| anyhow!("Embeddings model not initialized"))?;

        let mut documents = self.documents.write().await;
        let mut inverted_index = self.inverted_index.write().await;

        // Process each chunk
        for (idx, chunk) in chunks.iter().enumerate() {
            let chunk_id = format!("{}_{}", doc_id, idx);

            // Generate embeddings
            let embeddings = model.embed(vec![chunk.as_str()], None)?
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("Failed to generate embeddings"))?
                .to_vec();

            // Create document
            let document = Document {
                id: chunk_id.clone(),
                content: chunk.clone(),
                embeddings,
                metadata: metadata.clone(),
                timestamp: chrono::Utc::now().timestamp(),
                chunk_index: idx,
                total_chunks,
            };

            documents.insert(chunk_id.clone(), document);

            // Update inverted index for keyword search
            self.update_inverted_index(&chunk_id, chunk, &mut inverted_index);
        }

        // Persist to disk
        drop(documents);
        drop(inverted_index);
        self.save_index().await?;

        Ok(doc_id)
    }

    pub async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        // Ensure embeddings model is loaded (lazy load on first use)
        self.ensure_embeddings_model().await?;

        let config = self.config.read().await;
        let limit = limit.unwrap_or(config.max_results);

        // Generate query embedding
        let mut model_lock = self.embeddings_model.write().await;
        let model = model_lock.as_mut()
            .ok_or_else(|| anyhow!("Embeddings model not initialized"))?;

        let query_embedding = model.embed(vec![query], None)?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Failed to generate query embedding"))?
            .to_vec();

        let mut results = if config.enable_hybrid_search {
            // Hybrid search: combine vector and keyword search
            self.hybrid_search(query, &query_embedding, limit).await?
        } else {
            // Pure vector search
            self.vector_search(&query_embedding, limit).await?
        };

        // Apply reranking if enabled
        if config.enable_reranking && !results.is_empty() {
            results = self.rerank_results(query, results).await?;
        }

        Ok(results)
    }

    async fn vector_search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        let documents = self.documents.read().await;
        let config = self.config.read().await;
        let mut scores: Vec<(String, f32, Document)> = Vec::new();

        // Calculate similarity for all documents
        for (id, doc) in documents.iter() {
            let similarity = cosine_similarity(query_embedding, &doc.embeddings);

            if similarity >= config.similarity_threshold {
                scores.push((id.clone(), similarity, doc.clone()));
            }
        }

        // Sort by score, handling NaN values safely
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(limit);

        // Convert to SearchResult
        Ok(scores.into_iter().map(|(id, score, doc)| SearchResult {
            document_id: id,
            content: doc.content,
            score,
            metadata: doc.metadata,
            highlight: None,
            reasoning: None,
        }).collect())
    }

    async fn hybrid_search(
        &self,
        query: &str,
        query_embedding: &[f32],
        limit: usize
    ) -> Result<Vec<SearchResult>> {
        // Get vector search results
        let vector_results = self.vector_search(query_embedding, limit * 2).await?;

        // Get keyword search results
        let keyword_results = self.keyword_search(query, limit * 2).await?;

        // Merge and deduplicate results
        let mut merged_scores: HashMap<String, (f32, SearchResult)> = HashMap::new();

        // Add vector results with weight
        for result in vector_results {
            let weighted_score = result.score * 0.7; // 70% weight for vector search
            merged_scores.insert(
                result.document_id.clone(),
                (weighted_score, result)
            );
        }

        // Add or update with keyword results
        for result in keyword_results {
            let weighted_score = result.score * 0.3; // 30% weight for keyword search

            merged_scores.entry(result.document_id.clone())
                .and_modify(|e| e.0 += weighted_score)
                .or_insert((weighted_score, result));
        }

        // Sort by combined score
        let mut final_results: Vec<SearchResult> = merged_scores
            .into_iter()
            .map(|(_, (score, mut result))| {
                result.score = score;
                result
            })
            .collect();

        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        final_results.truncate(limit);

        Ok(final_results)
    }

    async fn keyword_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let documents = self.documents.read().await;
        let inverted_index = self.inverted_index.read().await;
        let mut scores: HashMap<String, f32> = HashMap::new();

        // Tokenize query
        let query_tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2) // Skip short words
            .map(|s| s.to_string())
            .collect();

        // Find documents containing query tokens
        for token in &query_tokens {
            if let Some(doc_ids) = inverted_index.get(token) {
                for doc_id in doc_ids {
                    *scores.entry(doc_id.clone()).or_insert(0.0) += 1.0;
                }
            }
        }

        // Normalize scores and create results
        let max_score = scores.values().cloned().fold(0.0f32, f32::max);
        let mut results: Vec<SearchResult> = Vec::new();

        if max_score > 0.0 {
            for (doc_id, score) in scores {
                if let Some(doc) = documents.get(&doc_id) {
                    results.push(SearchResult {
                        document_id: doc_id,
                        content: doc.content.clone(),
                        score: score / max_score,
                        metadata: doc.metadata.clone(),
                        highlight: self.generate_highlight(&doc.content, &query_tokens),
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
        // Cross-encoder style reranking
        let query_lower = query.to_lowercase();
        let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();

        for result in &mut results {
            let content_lower = result.content.to_lowercase();
            let mut boost = 0.0;

            // Boost for exact phrase match
            if content_lower.contains(&query_lower) {
                boost += 0.3;
            }

            // Boost for all query tokens present
            let all_tokens_present = query_tokens.iter()
                .all(|token| content_lower.contains(token));
            if all_tokens_present {
                boost += 0.2;
            }

            // Boost for token proximity
            if query_tokens.len() > 1 {
                let proximity_score = self.calculate_token_proximity(&content_lower, &query_tokens);
                boost += proximity_score * 0.1;
            }

            // Apply boost
            result.score = (result.score + boost).min(1.0);
        }

        // Re-sort after reranking, handling NaN values safely
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    fn calculate_token_proximity(&self, content: &str, tokens: &[&str]) -> f32 {
        if tokens.len() < 2 {
            return 0.0;
        }

        let mut positions: Vec<Vec<usize>> = Vec::new();

        // Find positions of each token
        for token in tokens {
            let mut token_positions = Vec::new();
            let mut start = 0;

            while let Some(pos) = content[start..].find(token) {
                token_positions.push(start + pos);
                start = start + pos + token.len();
            }

            if token_positions.is_empty() {
                return 0.0; // Token not found
            }

            positions.push(token_positions);
        }

        // Calculate minimum span containing all tokens
        let mut min_span = content.len();

        // Check all combinations
        for first_pos in &positions[0] {
            let mut max_pos = *first_pos;
            let mut min_pos = *first_pos;

            for token_positions in &positions[1..] {
                if let Some(closest) = token_positions.iter()
                    .min_by_key(|&&p| (*first_pos as i32 - p as i32).abs()) {
                    max_pos = max_pos.max(*closest);
                    min_pos = min_pos.min(*closest);
                }
            }

            let span = max_pos - min_pos;
            min_span = min_span.min(span);
        }

        // Convert to proximity score (closer = higher score)
        let max_distance = 100; // Maximum considered distance
        let proximity = 1.0 - (min_span.min(max_distance) as f32 / max_distance as f32);
        proximity
    }

    fn generate_highlight(&self, content: &str, query_tokens: &[String]) -> Option<String> {
        let content_lower = content.to_lowercase();
        let mut best_start = 0;
        let mut best_score = 0;

        // Find the best window containing most query tokens
        let window_size = 150;

        for i in 0..content.len().saturating_sub(window_size) {
            let window = &content_lower[i..std::cmp::min(i + window_size, content.len())];
            let score = query_tokens.iter()
                .filter(|token| window.contains(token.as_str()))
                .count();

            if score > best_score {
                best_score = score;
                best_start = i;
            }
        }

        if best_score > 0 {
            let end = std::cmp::min(best_start + window_size, content.len());
            let mut highlight = content[best_start..end].to_string();

            // Add ellipsis if truncated
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
        let config = self.config.read().await;
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();

        if words.is_empty() {
            return chunks;
        }

        let mut i = 0;
        while i < words.len() {
            let chunk_end = std::cmp::min(i + config.chunk_size, words.len());
            let chunk = words[i..chunk_end].join(" ");
            chunks.push(chunk);

            // Move forward with overlap
            if chunk_end < words.len() {
                i += config.chunk_size - config.chunk_overlap;
            } else {
                break;
            }
        }

        chunks
    }

    fn update_inverted_index(
        &self,
        doc_id: &str,
        content: &str,
        index: &mut HashMap<String, Vec<String>>
    ) {
        // Tokenize and index
        let tokens: Vec<String> = content
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2) // Skip short words
            .map(|s| s.to_string())
            .collect();

        for token in tokens {
            index.entry(token)
                .or_insert_with(Vec::new)
                .push(doc_id.to_string());
        }
    }

    // Cosine similarity now uses shared utility function from crate::utils

    #[allow(dead_code)]
    pub async fn delete_document(&self, doc_id: &str) -> Result<()> {
        let mut documents = self.documents.write().await;
        let mut inverted_index = self.inverted_index.write().await;

        // Remove all chunks for this document
        let keys_to_remove: Vec<String> = documents
            .keys()
            .filter(|k| k.starts_with(doc_id))
            .cloned()
            .collect();

        for key in &keys_to_remove {
            if let Some(doc) = documents.get(key) {
                // Remove from inverted index
                let tokens: Vec<String> = doc.content
                    .to_lowercase()
                    .split_whitespace()
                    .filter(|w| w.len() > 2)
                    .map(|s| s.to_string())
                    .collect();

                for token in tokens {
                    if let Some(doc_ids) = inverted_index.get_mut(&token) {
                        doc_ids.retain(|id| id != key);
                    }
                }
            }

            documents.remove(key);
        }

        drop(documents);
        drop(inverted_index);
        self.save_index().await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn clear_index(&self) -> Result<()> {
        let mut documents = self.documents.write().await;
        let mut inverted_index = self.inverted_index.write().await;

        documents.clear();
        inverted_index.clear();

        drop(documents);
        drop(inverted_index);
        self.save_index().await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_statistics(&self) -> Result<JsonValue> {
        let documents = self.documents.read().await;
        let inverted_index = self.inverted_index.read().await;
        let config = self.config.read().await;

        // Calculate unique documents (not chunks)
        let mut unique_docs = std::collections::HashSet::new();
        for key in documents.keys() {
            if let Some(doc_id) = key.split('_').next() {
                unique_docs.insert(doc_id);
            }
        }

        Ok(serde_json::json!({
            "total_chunks": documents.len(),
            "total_documents": unique_docs.len(),
            "total_tokens_indexed": inverted_index.len(),
            "index_size_bytes": self.estimate_index_size(&documents, &inverted_index),
            "config": {
                "chunk_size": config.chunk_size,
                "chunk_overlap": config.chunk_overlap,
                "embedding_model": config.embedding_model,
                "enable_reranking": config.enable_reranking,
                "enable_hybrid_search": config.enable_hybrid_search,
            }
        }))
    }

    #[allow(dead_code)]
    fn estimate_index_size(
        &self,
        documents: &HashMap<String, Document>,
        inverted_index: &HashMap<String, Vec<String>>
    ) -> usize {
        let doc_size: usize = documents.iter()
            .map(|(k, v)| {
                k.len() + v.content.len() +
                v.embeddings.len() * 4 + // f32 = 4 bytes
                v.metadata.to_string().len()
            })
            .sum();

        let index_size: usize = inverted_index.iter()
            .map(|(k, v)| k.len() + v.iter().map(|s| s.len()).sum::<usize>())
            .sum();

        doc_size + index_size
    }

    async fn save_index(&self) -> Result<()> {
        let documents = self.documents.read().await;
        let index_file = self.index_path.join("documents.json");

        let serialized = serde_json::to_string(&*documents)?;
        tokio::fs::write(index_file, serialized).await?;

        // Save inverted index
        let inverted_index = self.inverted_index.read().await;
        let inverted_file = self.index_path.join("inverted_index.json");

        let serialized_inverted = serde_json::to_string(&*inverted_index)?;
        tokio::fs::write(inverted_file, serialized_inverted).await?;

        Ok(())
    }

    async fn load_index(&self) -> Result<()> {
        // Load documents
        let index_file = self.index_path.join("documents.json");
        if index_file.exists() {
            let data = tokio::fs::read_to_string(&index_file).await?;
            let loaded_docs: HashMap<String, Document> = serde_json::from_str(&data)?;

            let mut documents = self.documents.write().await;
            *documents = loaded_docs;
        }

        // Load inverted index
        let inverted_file = self.index_path.join("inverted_index.json");
        if inverted_file.exists() {
            let data = tokio::fs::read_to_string(&inverted_file).await?;
            let loaded_index: HashMap<String, Vec<String>> = serde_json::from_str(&data)?;

            let mut inverted_index = self.inverted_index.write().await;
            *inverted_index = loaded_index;
        }

        Ok(())
    }

    pub async fn clear_cache(&self) -> Result<()> {
        // Clear in-memory embeddings cache
        let mut embeddings_model = self.embeddings_model.write().await;
        *embeddings_model = None;

        tracing::info!("RAG engine cache cleared");
        Ok(())
    }

    /// Generate augmented prompt for LLM with retrieved context
    ///
    /// This is the critical RAG function that combines retrieved documents
    /// with the user's query to create a context-aware prompt for the LLM.
    ///
    /// # Arguments
    /// * `query` - The user's original question
    /// * `limit` - Optional limit on number of documents to retrieve
    ///
    /// # Returns
    /// A fully formatted prompt string ready for LLM inference, containing:
    /// - System instruction to use only provided context
    /// - Retrieved document context with source markers
    /// - User's original query
    ///
    /// # Example
    /// ```rust
    /// let prompt = rag_engine.generate_augmented_prompt(
    ///     "What are the main clauses in the contract?",
    ///     Some(5)
    /// ).await?;
    /// // Use prompt with LLM: llm_manager.generate(&prompt, None).await?
    /// ```
    pub async fn generate_augmented_prompt(&self, query: &str, limit: Option<usize>) -> Result<String> {
        // Execute search to retrieve relevant document chunks
        let search_results = self.search(query, limit).await?;

        if search_results.is_empty() {
            // No relevant documents found - return prompt without context
            tracing::warn!("No relevant documents found for query: {}", &query[..query.len().min(50)]);

            return Ok(format!(
                "INSTRUCTION: Answer the following question to the best of your ability.\n\
                \n\
                QUESTION: {}\n\
                \n\
                ANSWER:",
                query
            ));
        }

        // Format context from retrieved documents
        let mut context_parts = Vec::new();

        for (idx, result) in search_results.iter().enumerate() {
            // Add source marker and content
            let source_marker = format!("--- SOURCE {} (Relevance: {:.2}) ---", idx + 1, result.score);

            // Include metadata if available
            let metadata_str = if let Some(obj) = result.metadata.as_object() {
                let meta_items: Vec<String> = obj.iter()
                    .filter_map(|(k, v)| {
                        if let Some(s) = v.as_str() {
                            Some(format!("{}: {}", k, s))
                        } else {
                            None
                        }
                    })
                    .collect();

                if !meta_items.is_empty() {
                    format!("\nMetadata: {}", meta_items.join(", "))
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            // Use highlight if available, otherwise use full content
            let content = if let Some(highlight) = &result.highlight {
                highlight.clone()
            } else {
                result.content.clone()
            };

            context_parts.push(format!(
                "{}{}\n\n{}",
                source_marker,
                metadata_str,
                content
            ));
        }

        let formatted_context = context_parts.join("\n\n");

        // Construct the final augmented prompt with clear instructions
        let augmented_prompt = format!(
            "SYSTEM INSTRUCTION: You are a legal AI assistant. Answer the user's question \
            using ONLY the information provided in the context below. If the context does \
            not contain sufficient information to answer the question, clearly state that \
            the available documents do not provide enough information.\n\
            \n\
            Do not make assumptions or use knowledge beyond the provided context. Be precise, \
            cite specific sources when possible, and maintain a professional tone.\n\
            \n\
            CONTEXT:\n\
            {}\n\
            \n\
            QUESTION: {}\n\
            \n\
            ANSWER (based solely on the context above):",
            formatted_context,
            query
        );

        tracing::info!(
            "Generated augmented prompt with {} sources ({} chars) for query: {}",
            search_results.len(),
            augmented_prompt.len(),
            &query[..query.len().min(50)]
        );

        Ok(augmented_prompt)
    }

}