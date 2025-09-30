use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tokio::fs;
use hf_hub::{api::tokio::Api, Repo, RepoType};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::model_manager::ModelManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub model_type: String,
    pub repo_id: String,
    pub model_file: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub context_length: usize,
    pub size_mb: u64,
    pub quantization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub temperature: f32,
    pub max_tokens: usize,
    pub top_p: f32,
    pub top_k: usize,
    pub repetition_penalty: f32,
    pub seed: Option<u64>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            max_tokens: 1024,
            top_p: 0.95,
            top_k: 40,
            repetition_penalty: 1.1,
            seed: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    NotDownloaded,
    Downloading { progress: f32 },
    Downloaded,
    Loading,
    Loaded,
    Failed(String),
}

pub struct LLMManagerV2 {
    models_registry: Arc<RwLock<HashMap<String, ModelConfig>>>,
    model_status: Arc<RwLock<HashMap<String, ModelStatus>>>,
    active_model: Arc<RwLock<Option<String>>>,
    models_dir: PathBuf,
    generation_config: Arc<RwLock<GenerationConfig>>,
    model_manager: Arc<ModelManager>,
}

impl LLMManagerV2 {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        let models_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("bear-ai-llm")
            .join("models");

        Self {
            models_registry: Arc::new(RwLock::new(HashMap::new())),
            model_status: Arc::new(RwLock::new(HashMap::new())),
            active_model: Arc::new(RwLock::new(None)),
            models_dir,
            generation_config: Arc::new(RwLock::new(GenerationConfig::default())),
            model_manager,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Create models directory
        fs::create_dir_all(&self.models_dir).await?;

        // Load model registry
        self.load_model_registry().await?;

        // Scan for already downloaded models
        self.scan_local_models().await?;

        Ok(())
    }

    async fn load_model_registry(&self) -> Result<()> {
        let default_models = vec![
            ModelConfig {
                name: "tinyllama-1.1b".to_string(),
                model_type: "llama".to_string(),
                repo_id: "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF".to_string(),
                model_file: "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf".to_string(),
                max_tokens: 1024,
                temperature: 0.8,
                context_length: 2048,
                size_mb: 638,
                quantization: "Q4_K_M".to_string(),
            },
            ModelConfig {
                name: "phi-2".to_string(),
                model_type: "phi".to_string(),
                repo_id: "TheBloke/phi-2-GGUF".to_string(),
                model_file: "phi-2.Q4_K_M.gguf".to_string(),
                max_tokens: 1024,
                temperature: 0.8,
                context_length: 2048,
                size_mb: 1600,
                quantization: "Q4_K_M".to_string(),
            },
            ModelConfig {
                name: "mistral-7b-instruct".to_string(),
                model_type: "mistral".to_string(),
                repo_id: "TheBloke/Mistral-7B-Instruct-v0.2-GGUF".to_string(),
                model_file: "mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string(),
                max_tokens: 2048,
                temperature: 0.8,
                context_length: 4096,
                size_mb: 4370,
                quantization: "Q4_K_M".to_string(),
            },
            ModelConfig {
                name: "llama2-7b-chat".to_string(),
                model_type: "llama".to_string(),
                repo_id: "TheBloke/Llama-2-7B-Chat-GGUF".to_string(),
                model_file: "llama-2-7b-chat.Q4_K_M.gguf".to_string(),
                max_tokens: 2048,
                temperature: 0.7,
                context_length: 4096,
                size_mb: 3830,
                quantization: "Q4_K_M".to_string(),
            },
        ];

        let mut registry = self.models_registry.write().await;
        let mut status = self.model_status.write().await;

        for model in default_models {
            status.insert(model.name.clone(), ModelStatus::NotDownloaded);
            registry.insert(model.name.clone(), model);
        }

        Ok(())
    }

    async fn scan_local_models(&self) -> Result<()> {
        let mut status = self.model_status.write().await;

        // Check which models are already downloaded
        for (name, _config) in self.models_registry.read().await.iter() {
            let model_dir = self.models_dir.join(name);
            if model_dir.exists() {
                // Check if the model file exists
                let model_files = fs::read_dir(&model_dir).await?;
                let mut has_model = false;

                let mut entries = model_files;
                while let Some(entry) = entries.next_entry().await? {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("gguf") {
                        has_model = true;
                        break;
                    }
                }

                if has_model {
                    status.insert(name.clone(), ModelStatus::Downloaded);
                }
            }
        }

        Ok(())
    }

    /// Consolidated method for ensuring a model is ready to use
    pub async fn ensure_model_ready(&self, model_name: &str) -> Result<()> {
        // Check if model exists in registry
        let registry = self.models_registry.read().await;
        if !registry.contains_key(model_name) {
            return Err(anyhow!("Model '{}' not found in registry", model_name));
        }
        drop(registry);

        // Check current status
        let status_map = self.model_status.read().await;
        let current_status = status_map.get(model_name).cloned()
            .unwrap_or(ModelStatus::NotDownloaded);
        drop(status_map);

        match current_status {
            ModelStatus::Loaded => {
                // Model is already loaded and ready
                return Ok(());
            }
            ModelStatus::Downloaded => {
                // Model is downloaded but not loaded
                self.load_model(model_name).await?;
            }
            ModelStatus::NotDownloaded => {
                // Need to download and load
                self.download_and_load(model_name).await?;
            }
            ModelStatus::Downloading { .. } => {
                return Err(anyhow!("Model is currently downloading"));
            }
            ModelStatus::Loading => {
                return Err(anyhow!("Model is currently loading"));
            }
            ModelStatus::Failed(err) => {
                return Err(anyhow!("Model failed to load: {}", err));
            }
        }

        Ok(())
    }

    /// Download and load a model in one operation
    async fn download_and_load(&self, model_name: &str) -> Result<()> {
        // Download first
        self.download_model(model_name).await?;

        // Then load
        self.load_model(model_name).await?;

        Ok(())
    }

    async fn download_model(&self, model_name: &str) -> Result<()> {
        let model_config = {
            let registry = self.models_registry.read().await;
            registry.get(model_name)
                .ok_or_else(|| anyhow!("Model not found: {}", model_name))?
                .clone()
        };

        // Update status to downloading
        {
            let mut status = self.model_status.write().await;
            status.insert(model_name.to_string(), ModelStatus::Downloading { progress: 0.0 });
        }

        // Use model_manager for actual download
        match self.model_manager.download_model(&model_config.repo_id).await {
            Ok(_) => {
                let mut status = self.model_status.write().await;
                status.insert(model_name.to_string(), ModelStatus::Downloaded);
                Ok(())
            }
            Err(e) => {
                let mut status = self.model_status.write().await;
                status.insert(model_name.to_string(), ModelStatus::Failed(e.to_string()));
                Err(e)
            }
        }
    }

    async fn load_model(&self, model_name: &str) -> Result<()> {
        // Update status to loading
        {
            let mut status = self.model_status.write().await;
            status.insert(model_name.to_string(), ModelStatus::Loading);
        }

        // Use model_manager for actual loading
        match self.model_manager.load_model(model_name).await {
            Ok(_) => {
                // Update active model
                let mut active = self.active_model.write().await;
                *active = Some(model_name.to_string());

                // Update status
                let mut status = self.model_status.write().await;
                status.insert(model_name.to_string(), ModelStatus::Loaded);

                Ok(())
            }
            Err(e) => {
                let mut status = self.model_status.write().await;
                status.insert(model_name.to_string(), ModelStatus::Failed(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn generate_response(&self, prompt: &str, config: Option<GenerationConfig>) -> Result<String> {
        let active_model = self.active_model.read().await;
        let model_name = active_model.as_ref()
            .ok_or_else(|| anyhow!("No model is currently loaded"))?;

        // Get generation config
        let gen_config = if let Some(config) = config {
            config
        } else {
            self.generation_config.read().await.clone()
        };

        // For now, return a placeholder
        // In production, this would use the actual inference engine
        Ok(format!(
            "Response from {} model with temperature {}: Processing '{}'...",
            model_name, gen_config.temperature, prompt
        ))
    }

    pub async fn list_models(&self) -> Vec<(String, ModelConfig, ModelStatus)> {
        let registry = self.models_registry.read().await;
        let status_map = self.model_status.read().await;

        registry.iter()
            .map(|(name, config)| {
                let status = status_map.get(name)
                    .cloned()
                    .unwrap_or(ModelStatus::NotDownloaded);
                (name.clone(), config.clone(), status)
            })
            .collect()
    }

    pub async fn get_active_model(&self) -> Option<String> {
        self.active_model.read().await.clone()
    }

    pub async fn unload_model(&self) -> Result<()> {
        let mut active = self.active_model.write().await;
        if let Some(model_name) = active.as_ref() {
            let mut status = self.model_status.write().await;
            status.insert(model_name.clone(), ModelStatus::Downloaded);
        }
        *active = None;
        Ok(())
    }

    pub async fn update_generation_config(&self, config: GenerationConfig) -> Result<()> {
        let mut gen_config = self.generation_config.write().await;
        *gen_config = config;
        Ok(())
    }

    pub async fn get_model_status(&self, model_name: &str) -> Option<ModelStatus> {
        let status_map = self.model_status.read().await;
        status_map.get(model_name).cloned()
    }

    pub async fn get_model_info(&self, model_name: &str) -> Option<ModelConfig> {
        let registry = self.models_registry.read().await;
        registry.get(model_name).cloned()
    }
}