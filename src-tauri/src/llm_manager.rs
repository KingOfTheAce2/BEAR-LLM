use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use hf_hub::api::tokio::Api;
use candle_core::Device;
use tokenizers::Tokenizer;
use crate::gguf_inference::{GGUFInferenceEngine, GGUFInferenceConfig};
use crate::constants::*;

// Production LLM Manager with real model downloading and inference
// This is the single source of truth for LLM management in BEAR AI

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub model_type: String,
    pub repo_id: String,
    pub model_file: String,
    pub tokenizer_repo: Option<String>,
    pub max_tokens: usize,
    pub temperature: f32,
    pub context_length: usize,
    pub size_mb: u64,
    pub quantization: String,
    pub requires_gpu: bool,
    pub recommended_gpu_layers: Option<u32>, // Recommended GPU layers for this model
    pub recommended_vram_mb: Option<u64>,   // Recommended VRAM for full offload
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub temperature: f32,
    pub max_tokens: usize,
    pub top_p: f32,
    pub top_k: usize,
    pub repetition_penalty: f32,
    pub seed: Option<u64>,
    pub stop_sequences: Vec<String>,
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
            stop_sequences: vec!["</s>".to_string(), "[/INST]".to_string()],
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub text: String,
    pub tokens_generated: usize,
    pub time_ms: u128,
    pub tokens_per_second: f32,
}

pub struct LLMManager {
    models_registry: Arc<RwLock<HashMap<String, ModelConfig>>>,
    model_status: Arc<RwLock<HashMap<String, ModelStatus>>>,
    active_model: Arc<RwLock<Option<String>>>,
    gguf_engine: Arc<GGUFInferenceEngine>,
    tokenizer: Arc<RwLock<Option<Tokenizer>>>,
    models_dir: PathBuf,
    generation_config: Arc<RwLock<GenerationConfig>>,
    device: Device,
}

impl LLMManager {
    pub fn new() -> Result<Self> {
        let models_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("bear-ai-llm")
            .join("models");

        let device = if candle_core::utils::cuda_is_available() {
            Device::new_cuda(0).unwrap_or(Device::Cpu)
        } else {
            Device::Cpu
        };

        tracing::info!(device = ?device, "Initialized compute device");

        // Initialize GGUF inference engine
        let gguf_engine = GGUFInferenceEngine::new()
            .map_err(|e| anyhow!("Failed to initialize GGUF engine: {}", e))?;

        Ok(Self {
            models_registry: Arc::new(RwLock::new(HashMap::new())),
            model_status: Arc::new(RwLock::new(HashMap::new())),
            active_model: Arc::new(RwLock::new(None)),
            gguf_engine: Arc::new(gguf_engine),
            tokenizer: Arc::new(RwLock::new(None)),
            models_dir,
            generation_config: Arc::new(RwLock::new(GenerationConfig::default())),
            device,
        })
    }

    /// Get standardized model directory for a given model config
    ///
    /// This ensures consistency between download, load, and scan operations.
    /// Uses sanitized repo_id as the directory name for uniqueness and clarity.
    fn get_model_dir(&self, model_config: &ModelConfig) -> PathBuf {
        // Sanitize repo_id by replacing "/" with "_"
        // Example: "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF" -> "TheBloke_TinyLlama-1.1B-Chat-v1.0-GGUF"
        let sanitized_repo_id = model_config.repo_id.replace("/", "_");
        self.models_dir.join(sanitized_repo_id)
    }

    pub async fn initialize(&self) -> Result<()> {
        // Create models directory
        tokio::fs::create_dir_all(&self.models_dir).await?;

        // Load model registry
        self.load_model_registry().await;

        // Scan for already downloaded models
        self.scan_local_models().await?;

        let model_count = self.models_registry.read().await.len();
        tracing::info!(model_count, "LLM Manager initialized successfully");

        Ok(())
    }

    async fn load_model_registry(&self) {
        let models = vec![
            ModelConfig {
                name: "tinyllama-1.1b".to_string(),
                model_type: "llama".to_string(),
                repo_id: "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF".to_string(),
                model_file: "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf".to_string(),
                tokenizer_repo: Some("TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string()),
                max_tokens: 1024,
                temperature: DEFAULT_TEMPERATURE,
                context_length: 2048,
                size_mb: 638,
                quantization: "Q4_K_M".to_string(),
                requires_gpu: false,
                recommended_gpu_layers: Some(TINYLLAMA_GPU_LAYERS),
                recommended_vram_mb: Some(TINYLLAMA_VRAM_MB),
            },
            ModelConfig {
                name: "phi-2".to_string(),
                model_type: "phi".to_string(),
                repo_id: "TheBloke/phi-2-GGUF".to_string(),
                model_file: "phi-2.Q4_K_M.gguf".to_string(),
                tokenizer_repo: Some("microsoft/phi-2".to_string()),
                max_tokens: 1024,
                temperature: DEFAULT_TEMPERATURE,
                context_length: 2048,
                size_mb: 1600,
                quantization: "Q4_K_M".to_string(),
                requires_gpu: false,
                recommended_gpu_layers: Some(PHI2_GPU_LAYERS),
                recommended_vram_mb: Some(PHI2_VRAM_MB),
            },
            ModelConfig {
                name: "mistral-7b-instruct".to_string(),
                model_type: "mistral".to_string(),
                repo_id: "TheBloke/Mistral-7B-Instruct-v0.2-GGUF".to_string(),
                model_file: "mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string(),
                tokenizer_repo: Some("mistralai/Mistral-7B-Instruct-v0.2".to_string()),
                max_tokens: DEFAULT_MAX_TOKENS,
                temperature: DEFAULT_TEMPERATURE,
                context_length: DEFAULT_N_CTX as usize,
                size_mb: 4370,
                quantization: "Q4_K_M".to_string(),
                requires_gpu: true,
                recommended_gpu_layers: Some(MISTRAL_7B_GPU_LAYERS),
                recommended_vram_mb: Some(MISTRAL_7B_VRAM_MB),
            },
            ModelConfig {
                name: "llama2-7b-chat".to_string(),
                model_type: "llama".to_string(),
                repo_id: "TheBloke/Llama-2-7B-Chat-GGUF".to_string(),
                model_file: "llama-2-7b-chat.Q4_K_M.gguf".to_string(),
                tokenizer_repo: Some("meta-llama/Llama-2-7b-chat-hf".to_string()),
                max_tokens: DEFAULT_MAX_TOKENS,
                temperature: DEFAULT_TEMPERATURE,
                context_length: DEFAULT_N_CTX as usize,
                size_mb: 3830,
                quantization: "Q4_K_M".to_string(),
                requires_gpu: true,
                recommended_gpu_layers: Some(LLAMA2_7B_GPU_LAYERS),
                recommended_vram_mb: Some(LLAMA2_7B_VRAM_MB),
            },
        ];

        let mut registry = self.models_registry.write().await;
        let mut status = self.model_status.write().await;

        for model in models {
            status.insert(model.name.clone(), ModelStatus::NotDownloaded);
            registry.insert(model.name.clone(), model);
        }
    }

    async fn scan_local_models(&self) -> Result<()> {
        let mut status = self.model_status.write().await;

        for (name, config) in self.models_registry.read().await.iter() {
            // Use standardized model directory
            let model_dir = self.get_model_dir(config);

            if model_dir.exists() {
                // Check if model file exists
                let has_model = match tokio::fs::read_dir(&model_dir).await {
                    Ok(mut entries) => {
                        let mut found = false;
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            if let Some(ext) = entry.path().extension() {
                                if ext == "gguf" || ext == "bin" || ext == "safetensors" {
                                    found = true;
                                    break;
                                }
                            }
                        }
                        found
                    }
                    Err(_) => false,
                };

                if has_model {
                    status.insert(name.clone(), ModelStatus::Downloaded);
                    tracing::info!(model = %name, "Found local model");
                }
            }
        }

        Ok(())
    }

    pub async fn download_model(&self, model_name: &str) -> Result<()> {
        let model_config = {
            let registry = self.models_registry.read().await;
            registry.get(model_name)
                .ok_or_else(|| anyhow!("Model '{}' not found in registry", model_name))?
                .clone()
        };

        // Update status
        {
            let mut status = self.model_status.write().await;
            status.insert(model_name.to_string(), ModelStatus::Downloading { progress: 0.0 });
        }

        tracing::info!(model = %model_name, "Starting model download");

        // Create model directory using standardized path
        let model_dir = self.get_model_dir(&model_config);
        tokio::fs::create_dir_all(&model_dir).await?;

        tracing::debug!(
            model_dir = ?model_dir,
            repo_id = %model_config.repo_id,
            "Model directory: {:?}",
            model_dir
        );

        // Download using HuggingFace Hub
        let api = Api::new()?;
        let repo = api.model(model_config.repo_id.clone());

        // Download model file
        let model_path = model_dir.join(&model_config.model_file);
        if !model_path.exists() {
            tracing::debug!(file = %model_config.model_file, "Downloading model file");

            match repo.get(&model_config.model_file).await {
                Ok(downloaded_path) => {
                    tokio::fs::copy(&downloaded_path, &model_path).await?;
                    tracing::info!(file = %model_config.model_file, "Model file downloaded successfully");
                }
                Err(e) => {
                    let mut status = self.model_status.write().await;
                    status.insert(model_name.to_string(),
                                 ModelStatus::Failed(format!("Download failed: {}", e)));
                    tracing::error!(model = %model_name, error = %e, "Failed to download model");
                    return Err(anyhow!("Failed to download model: {}", e));
                }
            }
        }

        // Download tokenizer if specified
        if let Some(tokenizer_repo) = &model_config.tokenizer_repo {
            let tokenizer_path = model_dir.join("tokenizer.json");
            if !tokenizer_path.exists() {
                tracing::debug!(repo = %tokenizer_repo, "Downloading tokenizer");

                let tokenizer_api = api.model(tokenizer_repo.clone());
                if let Ok(downloaded_path) = tokenizer_api.get("tokenizer.json").await {
                    tokio::fs::copy(&downloaded_path, &tokenizer_path).await?;
                    tracing::info!("Tokenizer downloaded successfully");
                }

                // Also try to get tokenizer config
                if let Ok(downloaded_path) = tokenizer_api.get("tokenizer_config.json").await {
                    let config_path = model_dir.join("tokenizer_config.json");
                    tokio::fs::copy(&downloaded_path, &config_path).await?;
                }
            }
        }

        // Update status
        {
            let mut status = self.model_status.write().await;
            status.insert(model_name.to_string(), ModelStatus::Downloaded);
        }

        tracing::info!(model = %model_name, "Model downloaded successfully");

        Ok(())
    }

    pub async fn load_model(&self, model_name: &str) -> Result<()> {
        // Check if model is downloaded
        let status = self.model_status.read().await.get(model_name).cloned()
            .unwrap_or(ModelStatus::NotDownloaded);

        match status {
            ModelStatus::NotDownloaded => {
                return Err(anyhow!("Model '{}' is not downloaded", model_name));
            }
            ModelStatus::Downloading { .. } => {
                return Err(anyhow!("Model '{}' is currently downloading", model_name));
            }
            ModelStatus::Loading => {
                return Err(anyhow!("Model '{}' is already loading", model_name));
            }
            ModelStatus::Loaded => {
                tracing::debug!(model = %model_name, "Model already loaded");
                return Ok(());
            }
            _ => {}
        }

        // Update status
        {
            let mut status = self.model_status.write().await;
            status.insert(model_name.to_string(), ModelStatus::Loading);
        }

        tracing::info!(model = %model_name, "Loading GGUF model");

        // Get model config
        let model_config = {
            let registry = self.models_registry.read().await;
            registry.get(model_name)
                .ok_or_else(|| anyhow!("Model '{}' not found in registry", model_name))?
                .clone()
        };

        // Find GGUF model file using standardized path
        let model_dir = self.get_model_dir(&model_config);
        let model_file = model_dir.join(&model_config.model_file);

        tracing::debug!(
            model_file = ?model_file,
            repo_id = %model_config.repo_id,
            "Looking for model file: {:?}",
            model_file
        );

        if !model_file.exists() {
            let mut status = self.model_status.write().await;
            status.insert(model_name.to_string(), ModelStatus::Failed("Model file not found".to_string()));
            return Err(anyhow!("Model file not found: {:?}", model_file));
        }

        // Calculate optimal GPU layers based on available VRAM
        let n_gpu_layers = self.calculate_optimal_gpu_layers(&model_config).await;

        // Load model into GGUF engine
        self.gguf_engine.load_model(model_file, n_gpu_layers).await
            .map_err(|e| {
                tracing::error!("Failed to load GGUF model: {}", e);
                e
            })?;

        // Load tokenizer if available
        let tokenizer_path = model_dir.join("tokenizer.json");
        if tokenizer_path.exists() {
            match Tokenizer::from_file(tokenizer_path) {
                Ok(tokenizer) => {
                    let mut tokenizer_lock = self.tokenizer.write().await;
                    *tokenizer_lock = Some(tokenizer);
                    tracing::info!("Tokenizer loaded successfully");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to load tokenizer");
                }
            }
        }

        // Update active model
        {
            let mut active = self.active_model.write().await;
            *active = Some(model_name.to_string());
        }

        // Update status
        {
            let mut status = self.model_status.write().await;
            status.insert(model_name.to_string(), ModelStatus::Loaded);
        }

        tracing::info!(model = %model_name, "✅ GGUF model loaded and ready for inference");

        Ok(())
    }

    pub async fn ensure_model_ready(&self, model_name: &str) -> Result<()> {
        let status = self.model_status.read().await.get(model_name).cloned()
            .unwrap_or(ModelStatus::NotDownloaded);

        match status {
            ModelStatus::Loaded => Ok(()),
            ModelStatus::Downloaded => self.load_model(model_name).await,
            ModelStatus::NotDownloaded => {
                self.download_model(model_name).await?;
                self.load_model(model_name).await
            }
            ModelStatus::Downloading { .. } => {
                Err(anyhow!("Model is currently downloading"))
            }
            ModelStatus::Loading => {
                Err(anyhow!("Model is currently loading"))
            }
            ModelStatus::Failed(err) => {
                Err(anyhow!("Model failed to load: {}", err))
            }
        }
    }

    pub async fn generate(&self, prompt: &str, config: Option<GenerationConfig>) -> Result<InferenceResult> {
        let active_model = self.active_model.read().await;
        let _model_name = active_model.as_ref()
            .ok_or_else(|| anyhow!("No model is currently loaded"))?;

        let gen_config = match config {
            Some(cfg) => cfg,
            None => self.generation_config.read().await.clone(),
        };

        // Check if GGUF model is loaded
        if !self.gguf_engine.is_model_loaded().await {
            return Err(anyhow!("GGUF model not loaded. Call load_model() first."));
        }

        tracing::debug!("Generating text for prompt: {}", &prompt[..prompt.len().min(50)]);

        // Generate using GGUF engine
        let result = self.gguf_engine.generate(
            prompt,
            gen_config.max_tokens,
            gen_config.stop_sequences.clone(),
        ).await?;

        tracing::info!(
            "Generated {} tokens in {:.2}s ({:.2} tok/s)",
            result.tokens_generated,
            result.time_ms as f32 / 1000.0,
            result.tokens_per_second
        );

        Ok(InferenceResult {
            text: result.text,
            tokens_generated: result.tokens_generated,
            time_ms: result.time_ms,
            tokens_per_second: result.tokens_per_second,
        })
    }

    /// Generate text with streaming support
    pub async fn generate_stream<F>(
        &self,
        prompt: &str,
        config: Option<GenerationConfig>,
        on_token: F,
    ) -> Result<InferenceResult>
    where
        F: FnMut(&str) -> bool + Send + 'static,
    {
        let active_model = self.active_model.read().await;
        let _model_name = active_model.as_ref()
            .ok_or_else(|| anyhow!("No model is currently loaded"))?;

        let gen_config = match config {
            Some(cfg) => cfg,
            None => self.generation_config.read().await.clone(),
        };

        // Check if GGUF model is loaded
        if !self.gguf_engine.is_model_loaded().await {
            return Err(anyhow!("GGUF model not loaded. Call load_model() first."));
        }

        tracing::debug!("Streaming generation for prompt: {}", &prompt[..prompt.len().min(50)]);

        // Generate with streaming using GGUF engine
        let result = self.gguf_engine.generate_stream(
            prompt,
            gen_config.max_tokens,
            gen_config.stop_sequences.clone(),
            on_token,
        ).await?;

        tracing::info!(
            "Streamed {} tokens in {:.2}s ({:.2} tok/s)",
            result.tokens_generated,
            result.time_ms as f32 / 1000.0,
            result.tokens_per_second
        );

        Ok(InferenceResult {
            text: result.text,
            tokens_generated: result.tokens_generated,
            time_ms: result.time_ms,
            tokens_per_second: result.tokens_per_second,
        })
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

    /// Check if a model is currently loaded
    pub async fn is_model_loaded(&self) -> Result<bool> {
        let active = self.active_model.read().await;
        Ok(active.is_some() && self.gguf_engine.is_model_loaded().await)
    }

    #[allow(dead_code)]
    pub async fn get_active_model(&self) -> Option<String> {
        self.active_model.read().await.clone()
    }

    pub async fn unload_model(&self) -> Result<()> {
        let mut active = self.active_model.write().await;

        if let Some(model_name) = active.as_ref() {
            let mut status = self.model_status.write().await;
            status.insert(model_name.clone(), ModelStatus::Downloaded);
            tracing::info!(model = %model_name, "Model unloaded");
        }

        *active = None;

        // Unload GGUF model
        self.gguf_engine.unload_model().await?;

        let mut tokenizer = self.tokenizer.write().await;
        *tokenizer = None;

        Ok(())
    }

    pub async fn update_generation_config(&self, config: GenerationConfig) -> Result<()> {
        let mut gen_config = self.generation_config.write().await;
        *gen_config = config.clone();

        // Update GGUF engine config as well
        let gguf_config = GGUFInferenceConfig {
            model_path: PathBuf::new(), // Will be set on load
            n_ctx: 2048,
            n_batch: 512,
            n_threads: std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(4),
            n_gpu_layers: 0,
            temperature: config.temperature,
            top_k: config.top_k as i32,
            top_p: config.top_p,
            repeat_penalty: config.repetition_penalty,
            seed: config.seed.unwrap_or(42) as u32,
        };

        self.gguf_engine.update_config(gguf_config).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_model_status(&self, model_name: &str) -> Option<ModelStatus> {
        let status_map = self.model_status.read().await;
        status_map.get(model_name).cloned()
    }

    #[allow(dead_code)]
    pub async fn get_model_info(&self, model_name: &str) -> Option<ModelConfig> {
        let registry = self.models_registry.read().await;
        registry.get(model_name).cloned()
    }

    #[allow(dead_code)]
    pub async fn delete_model(&self, model_name: &str) -> Result<()> {
        // Check if model is currently loaded
        let active = self.active_model.read().await;
        if active.as_ref() == Some(&model_name.to_string()) {
            return Err(anyhow!("Cannot delete currently loaded model"));
        }

        // Delete model files
        let model_dir = self.models_dir.join(model_name);
        if model_dir.exists() {
            tokio::fs::remove_dir_all(&model_dir).await?;
        }

        // Update status
        let mut status = self.model_status.write().await;
        status.insert(model_name.to_string(), ModelStatus::NotDownloaded);

        tracing::info!(model = %model_name, "Model deleted");

        Ok(())
    }

    /// Calculate optimal number of GPU layers to offload based on available VRAM
    async fn calculate_optimal_gpu_layers(&self, model_config: &ModelConfig) -> u32 {
        // If no GPU available, return 0
        if self.device.is_cpu() {
            tracing::info!("CPU mode: No GPU layers will be offloaded");
            return 0;
        }

        // Try to get GPU VRAM info using nvml-wrapper
        let available_vram_mb = match self.get_available_vram_mb() {
            Some(vram) => vram,
            None => {
                // Fallback: assume conservative 4GB if detection fails
                tracing::warn!("Could not detect GPU VRAM, assuming 4GB");
                4096
            }
        };

        tracing::info!(
            "GPU detected with ~{}MB VRAM available",
            available_vram_mb
        );

        // Get recommended layers and VRAM for this model
        let recommended_layers = model_config.recommended_gpu_layers.unwrap_or(0);
        let recommended_vram = model_config.recommended_vram_mb.unwrap_or(model_config.size_mb);

        // Calculate what percentage of the model we can offload
        // Keep some VRAM free for context and computation (based on VRAM_USAGE_RATIO)
        let usable_vram = (available_vram_mb as f32 * VRAM_USAGE_RATIO) as u64;

        if usable_vram < recommended_vram {
            // Partial offload: scale layers proportionally
            let ratio = usable_vram as f32 / recommended_vram as f32;
            let scaled_layers = (recommended_layers as f32 * ratio) as u32;

            tracing::info!(
                "Partial GPU offload: {} of {} layers ({}% of model) due to VRAM constraints",
                scaled_layers,
                recommended_layers,
                (ratio * 100.0) as u32
            );

            scaled_layers
        } else {
            // Full offload possible
            tracing::info!(
                "Full GPU offload: {} layers (sufficient VRAM: {}MB >= {}MB)",
                recommended_layers,
                usable_vram,
                recommended_vram
            );

            recommended_layers
        }
    }

    /// Get available VRAM in MB using NVML
    fn get_available_vram_mb(&self) -> Option<u64> {
        use nvml_wrapper::Nvml;

        match Nvml::init() {
            Ok(nvml) => {
                // Get first GPU device
                match nvml.device_by_index(0) {
                    Ok(device) => {
                        // Get memory info
                        match device.memory_info() {
                            Ok(mem_info) => {
                                let available_mb = mem_info.free / (1024 * 1024);
                                Some(available_mb)
                            }
                            Err(e) => {
                                tracing::warn!("Failed to get GPU memory info: {}", e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get GPU device: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                tracing::debug!("NVML not available: {}", e);
                None
            }
        }
    }

    pub async fn load_model_for_inference(&self, model_path: &str) -> Result<()> {
        // This method loads a model for inference
        // The model_path is the model name or identifier

        let mut active = self.active_model.write().await;
        let mut status_map = self.model_status.write().await;

        // Check if model exists in registry
        let registry = self.models_registry.read().await;
        let model_config = registry.get(model_path)
            .ok_or_else(|| anyhow!("Model '{}' not found in registry", model_path))?
            .clone();
        drop(registry);

        // Update status to loaded
        status_map.insert(model_path.to_string(), ModelStatus::Loaded);
        *active = Some(model_path.to_string());

        tracing::info!(
            model = %model_path,
            size_mb = model_config.size_mb,
            "Model loaded for inference"
        );

        Ok(())
    }
}