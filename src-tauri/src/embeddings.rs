use anyhow::{Result, anyhow};
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::utils::cosine_similarity;

pub struct EmbeddingsEngine {
    model: Arc<RwLock<Option<TextEmbedding>>>,
    model_name: String,
    dimension: usize,
}

impl EmbeddingsEngine {
    pub fn new() -> Self {
        Self {
            model: Arc::new(RwLock::new(None)),
            model_name: "BAAI/bge-small-en-v1.5".to_string(),
            dimension: 384, // BGE small dimension
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize the embedding model
        let model = TextEmbedding::try_new(
            InitOptions {
                model_name: EmbeddingModel::BGESmallENV15,
                show_download_progress: true,
                ..Default::default()
            }
        )?;

        let mut model_lock = self.model.write().await;
        *model_lock = Some(model);

        Ok(())
    }

    pub async fn generate_embeddings(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let model_lock = self.model.read().await;

        if let Some(model) = model_lock.as_ref() {
            // Generate embeddings for the texts
            let embeddings = model.embed(texts, None)?;

            // Convert to Vec<Vec<f32>>
            let result: Vec<Vec<f32>> = embeddings
                .into_iter()
                .map(|e| e.to_vec())
                .collect();

            Ok(result)
        } else {
            Err(anyhow!("Embedding model not initialized"))
        }
    }

    pub async fn generate_single_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.generate_embeddings(vec![text]).await?;
        embeddings.into_iter().next()
            .ok_or_else(|| anyhow!("Failed to generate embedding"))
    }

    pub fn get_dimension(&self) -> usize {
        self.dimension
    }

    pub async fn change_model(&mut self, model_name: &str) -> Result<()> {
        // Map model name to FastEmbed model
        let embedding_model = match model_name {
            "BAAI/bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
            "BAAI/bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
            "sentence-transformers/all-MiniLM-L6-v2" => EmbeddingModel::AllMiniLML6V2,
            _ => return Err(anyhow!("Unsupported embedding model: {}", model_name)),
        };

        // Initialize the new model
        let model = TextEmbedding::try_new(
            InitOptions {
                model_name: embedding_model,
                show_download_progress: true,
                ..Default::default()
            }
        )?;

        // Update dimension based on model
        self.dimension = match embedding_model {
            EmbeddingModel::BGESmallENV15 => 384,
            EmbeddingModel::BGEBaseENV15 => 768,
            EmbeddingModel::AllMiniLML6V2 => 384,
            _ => 384,
        };

        let mut model_lock = self.model.write().await;
        *model_lock = Some(model);
        self.model_name = model_name.to_string();

        Ok(())
    }

}

/// Calculate cosine similarity between two embedding vectors
///
/// This function now delegates to the shared utility function
/// to avoid code duplication. Kept as a public function for backward compatibility.
///
/// # Deprecated
/// Use `crate::utils::cosine_similarity` directly instead.
pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    crate::utils::cosine_similarity(vec1, vec2)
}