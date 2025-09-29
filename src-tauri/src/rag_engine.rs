use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: JsonValue,
    pub embeddings: Vec<f32>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SearchResult {
    pub document_id: String,
    pub content: String,
    pub score: f32,
    pub metadata: JsonValue,
}

pub struct RAGEngine {
    documents: HashMap<String, Document>,
    index_path: PathBuf,
    embedding_dim: usize,
    chunk_size: usize,
    chunk_overlap: usize,
}

#[allow(dead_code)]
impl RAGEngine {
    pub fn new() -> Self {
        let index_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("legal-ai-assistant")
            .join("rag_index");

        Self {
            documents: HashMap::new(),
            index_path,
            embedding_dim: 384,
            chunk_size: 512,
            chunk_overlap: 50,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        tokio::fs::create_dir_all(&self.index_path).await?;
        self.load_index().await?;
        Ok(())
    }

    pub async fn add_document(&mut self, content: &str, metadata: JsonValue) -> Result<String> {
        let doc_id = Uuid::new_v4().to_string();
        let chunks = self.chunk_text(content);

        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_id = format!("{}_{}", doc_id, i);
            let embeddings = self.generate_embeddings(chunk).await?;

            let document = Document {
                id: chunk_id.clone(),
                content: chunk.clone(),
                metadata: metadata.clone(),
                embeddings,
                timestamp: chrono::Utc::now().timestamp(),
            };

            self.documents.insert(chunk_id, document);
        }

        self.save_index().await?;
        Ok(doc_id)
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<JsonValue>> {
        let query_embedding = self.generate_embeddings(query).await?;
        let mut results: Vec<(String, f32, Document)> = Vec::new();

        for (id, doc) in &self.documents {
            let score = self.cosine_similarity(&query_embedding, &doc.embeddings);
            results.push((id.clone(), score, doc.clone()));
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);

        let search_results: Vec<JsonValue> = results
            .into_iter()
            .map(|(id, score, doc)| {
                serde_json::json!({
                    "id": id,
                    "content": doc.content,
                    "score": score,
                    "metadata": doc.metadata,
                })
            })
            .collect();

        Ok(search_results)
    }

    pub async fn agentic_search(&self, query: &str, context: &str) -> Result<Vec<JsonValue>> {
        let enhanced_query = format!("{} Context: {}", query, context);
        let mut results = self.search(&enhanced_query, 10).await?;

        results = self.rerank_results(results, query).await?;

        let reasoning = self.generate_reasoning(&results, query).await?;
        results.push(serde_json::json!({
            "type": "reasoning",
            "content": reasoning,
        }));

        Ok(results)
    }

    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        let mut embeddings = vec![0.0; self.embedding_dim];
        for (i, char) in text.chars().enumerate().take(self.embedding_dim) {
            embeddings[i % self.embedding_dim] += (char as u32) as f32 / 1000.0;
        }

        let norm: f32 = embeddings.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embeddings {
                *val /= norm;
            }
        }

        Ok(embeddings)
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a * norm_b > 0.0 {
            dot_product / (norm_a * norm_b)
        } else {
            0.0
        }
    }

    fn chunk_text(&self, text: &str) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();

        for i in (0..words.len()).step_by(self.chunk_size - self.chunk_overlap) {
            let end = std::cmp::min(i + self.chunk_size, words.len());
            let chunk = words[i..end].join(" ");
            chunks.push(chunk);
        }

        if chunks.is_empty() {
            chunks.push(text.to_string());
        }

        chunks
    }

    async fn rerank_results(&self, results: Vec<JsonValue>, _query: &str) -> Result<Vec<JsonValue>> {
        let mut reranked = results;
        reranked.sort_by(|a, b| {
            let score_a = a["score"].as_f64().unwrap_or(0.0);
            let score_b = b["score"].as_f64().unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap()
        });

        Ok(reranked)
    }

    async fn generate_reasoning(&self, results: &[JsonValue], query: &str) -> Result<String> {
        let reasoning = format!(
            "Based on the query '{}', I found {} relevant documents. \
             The top results suggest that the answer relates to the following key points...",
            query,
            results.len()
        );

        Ok(reasoning)
    }

    async fn save_index(&self) -> Result<()> {
        let index_file = self.index_path.join("documents.json");
        let json = serde_json::to_string(&self.documents)?;
        tokio::fs::write(index_file, json).await?;
        Ok(())
    }

    async fn load_index(&mut self) -> Result<()> {
        let index_file = self.index_path.join("documents.json");
        if index_file.exists() {
            let json = tokio::fs::read_to_string(index_file).await?;
            self.documents = serde_json::from_str(&json)?;
        }
        Ok(())
    }

    pub async fn clear_index(&mut self) -> Result<()> {
        self.documents.clear();
        self.save_index().await?;
        Ok(())
    }

    pub fn get_document_count(&self) -> usize {
        self.documents.len()
    }
}