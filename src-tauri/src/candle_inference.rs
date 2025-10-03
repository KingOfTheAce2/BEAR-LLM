use crate::constants::*;
use anyhow::{anyhow, Result};
use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama as llama;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::RwLock;

/// Pure Rust GGUF inference engine using Candle
/// No C++ dependencies - reliable cross-platform builds
/// Supports GGUF/GGML quantized models with GPU acceleration

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGUFInferenceConfig {
    pub model_path: PathBuf,
    pub n_ctx: u32,          // Context size
    pub n_batch: u32,        // Batch size (not used by Candle, kept for API compatibility)
    pub n_threads: u32,      // CPU threads (Candle uses Rayon internally)
    pub n_gpu_layers: u32,   // Ignored - Candle auto-detects GPU
    pub temperature: f32,    // Sampling temperature
    pub top_k: i32,          // Top-k sampling
    pub top_p: f32,          // Top-p (nucleus) sampling
    pub repeat_penalty: f32, // Repetition penalty
    pub seed: u32,           // Random seed for reproducibility
}

impl Default for GGUFInferenceConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            n_ctx: DEFAULT_N_CTX,
            n_batch: DEFAULT_N_BATCH,
            n_threads: std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(CPU_THREAD_POOL_SIZE as u32),
            n_gpu_layers: 0,
            temperature: DEFAULT_TEMPERATURE,
            top_k: DEFAULT_TOP_K,
            top_p: DEFAULT_TOP_P,
            repeat_penalty: DEFAULT_REPEAT_PENALTY,
            seed: 42,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub text: String,
    pub tokens_generated: usize,
    pub time_ms: u128,
    pub tokens_per_second: f32,
    pub stop_reason: StopReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopReason {
    MaxTokens,
    StopSequence,
    EndOfText,
}

pub struct GGUFInferenceEngine {
    device: Device,
    model: Arc<RwLock<Option<llama::ModelWeights>>>,
    tokenizer: Arc<RwLock<Option<Tokenizer>>>,
    config: Arc<RwLock<GGUFInferenceConfig>>,
}

impl GGUFInferenceEngine {
    /// Create a new Candle-based inference engine
    pub fn new() -> Result<Self> {
        // Auto-detect best device (CUDA > Metal > CPU)
        let device = if candle_core::utils::cuda_is_available() {
            Device::new_cuda(0)?
        } else if candle_core::utils::metal_is_available() {
            Device::new_metal(0)?
        } else {
            Device::Cpu
        };

        tracing::info!("Candle inference engine initialized on device: {:?}", device);

        Ok(Self {
            device,
            model: Arc::new(RwLock::new(None)),
            tokenizer: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(GGUFInferenceConfig::default())),
        })
    }

    /// Load a GGUF model from file path
    pub async fn load_model(&self, model_path: impl AsRef<Path>, _n_gpu_layers: u32) -> Result<()> {
        let path = model_path.as_ref();

        if !path.exists() {
            return Err(anyhow!("Model file not found: {:?}", path));
        }

        tracing::info!("Loading GGUF model from: {:?}", path);

        // Load quantized model (GGUF format)
        use candle_transformers::quantized_var_builder::VarBuilder;

        let mut file = std::fs::File::open(path)?;
        let model = llama::ModelWeights::from_gguf(&mut file, &self.device)?;

        // Try to load tokenizer from same directory or HuggingFace cache
        let tokenizer_path = path.parent().map(|p| p.join("tokenizer.json"));
        let tokenizer = if let Some(tok_path) = tokenizer_path {
            if tok_path.exists() {
                Tokenizer::from_file(&tok_path)
                    .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?
            } else {
                // Fallback: create basic tokenizer
                tracing::warn!("Tokenizer not found at {:?}, using fallback", tok_path);
                self.create_fallback_tokenizer()?
            }
        } else {
            self.create_fallback_tokenizer()?
        };

        let mut model_lock = self.model.write().await;
        *model_lock = Some(model);

        let mut tokenizer_lock = self.tokenizer.write().await;
        *tokenizer_lock = Some(tokenizer);

        let mut config = self.config.write().await;
        config.model_path = path.to_path_buf();

        tracing::info!("✅ GGUF model loaded successfully with Candle");
        Ok(())
    }

    /// Check if a model is currently loaded
    pub async fn is_model_loaded(&self) -> bool {
        self.model.read().await.is_some()
    }

    /// Unload the current model
    pub async fn unload_model(&self) -> Result<()> {
        let mut model_lock = self.model.write().await;
        *model_lock = None;

        let mut tokenizer_lock = self.tokenizer.write().await;
        *tokenizer_lock = None;

        tracing::info!("Model unloaded");
        Ok(())
    }

    /// Generate text from a prompt
    pub async fn generate(
        &self,
        prompt: &str,
        max_tokens: usize,
        stop_sequences: Vec<String>,
    ) -> Result<GenerationResult> {
        let mut model_lock = self.model.write().await;
        let model = model_lock
            .as_mut()
            .ok_or_else(|| anyhow!("No model loaded. Call load_model() first."))?;

        let tokenizer_lock = self.tokenizer.read().await;
        let tokenizer = tokenizer_lock
            .as_ref()
            .ok_or_else(|| anyhow!("No tokenizer loaded"))?;

        let config = self.config.read().await.clone();

        // Tokenize prompt
        let encoding = tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

        let mut tokens = encoding.get_ids().to_vec();

        tracing::debug!("Prompt tokenized: {} tokens", tokens.len());

        // SECURITY: Validate context size
        let min_required_context = max_tokens + TOKEN_OVERFLOW_SAFETY_MARGIN;
        if min_required_context >= config.n_ctx as usize {
            return Err(anyhow!(
                "Insufficient context for generation:\n\
                - Requested max_tokens: {}\n\
                - Safety margin: {}\n\
                - Total required: {}\n\
                - Available context (n_ctx): {}",
                max_tokens,
                TOKEN_OVERFLOW_SAFETY_MARGIN,
                min_required_context,
                config.n_ctx
            ));
        }

        // Truncate if needed
        let max_prompt_tokens = (config.n_ctx as usize)
            .saturating_sub(max_tokens)
            .saturating_sub(TOKEN_OVERFLOW_SAFETY_MARGIN);

        if tokens.len() > max_prompt_tokens {
            tracing::warn!(
                "Prompt exceeds context limit. Truncating from {} to {} tokens",
                tokens.len(),
                max_prompt_tokens
            );
            tokens.truncate(max_prompt_tokens);
        }

        // Tokens are ready for inference (no explicit tensor conversion needed for quantized models)

        // Generation loop
        let start_time = std::time::Instant::now();
        let mut generated_text = String::new();
        let mut tokens_generated = 0;
        let mut stop_reason = StopReason::MaxTokens;
        let mut all_tokens = tokens.clone();

        for _ in 0..max_tokens {
            // Forward pass
            let logits = model.forward(&Tensor::new(&all_tokens[..], &self.device)?, all_tokens.len() - 1)?;

            // Sample next token with temperature, top-k, top-p
            let next_token = self.sample_token(&logits, &config)?;

            // Check for end of text (EOS token is typically 2 for LLaMA)
            if next_token == 2 {
                stop_reason = StopReason::EndOfText;
                break;
            }

            all_tokens.push(next_token);

            // Decode token to text
            let piece = tokenizer
                .decode(&[next_token], false)
                .map_err(|e| anyhow!("Failed to decode token: {}", e))?;

            generated_text.push_str(&piece);
            tokens_generated += 1;

            // Check for stop sequences
            if let Some((_matched_seq, pos)) = self.find_stop_sequence(&generated_text, &stop_sequences) {
                stop_reason = StopReason::StopSequence;
                generated_text.truncate(pos);
                break;
            }
        }

        let elapsed = start_time.elapsed();
        let tokens_per_second = if elapsed.as_secs_f32() > 0.0 {
            tokens_generated as f32 / elapsed.as_secs_f32()
        } else {
            0.0
        };

        tracing::info!(
            "Generated {} tokens in {:.2}s ({:.2} tok/s)",
            tokens_generated,
            elapsed.as_secs_f32(),
            tokens_per_second
        );

        Ok(GenerationResult {
            text: generated_text,
            tokens_generated,
            time_ms: elapsed.as_millis(),
            tokens_per_second,
            stop_reason,
        })
    }

    /// Generate text with streaming support
    pub async fn generate_stream<F>(
        &self,
        prompt: &str,
        max_tokens: usize,
        stop_sequences: Vec<String>,
        mut on_token: F,
    ) -> Result<GenerationResult>
    where
        F: FnMut(&str) -> bool,
    {
        let mut model_lock = self.model.write().await;
        let model = model_lock
            .as_mut()
            .ok_or_else(|| anyhow!("No model loaded"))?;

        let tokenizer_lock = self.tokenizer.read().await;
        let tokenizer = tokenizer_lock
            .as_ref()
            .ok_or_else(|| anyhow!("No tokenizer loaded"))?;

        let config = self.config.read().await.clone();

        let encoding = tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

        let mut tokens = encoding.get_ids().to_vec();

        let max_prompt_tokens = (config.n_ctx as usize)
            .saturating_sub(max_tokens)
            .saturating_sub(TOKEN_OVERFLOW_SAFETY_MARGIN);

        if tokens.len() > max_prompt_tokens {
            tokens.truncate(max_prompt_tokens);
        }

        let start_time = std::time::Instant::now();
        let mut generated_text = String::new();
        let mut tokens_generated = 0;
        let mut stop_reason = StopReason::MaxTokens;
        let mut all_tokens = tokens.clone();
        let mut user_stopped = false;

        for _ in 0..max_tokens {
            let logits = model.forward(&Tensor::new(&all_tokens[..], &self.device)?, all_tokens.len() - 1)?;
            let next_token = self.sample_token(&logits, &config)?;

            if next_token == 2 {
                stop_reason = StopReason::EndOfText;
                break;
            }

            all_tokens.push(next_token);

            let piece = tokenizer
                .decode(&[next_token], false)
                .map_err(|e| anyhow!("Failed to decode token: {}", e))?;

            generated_text.push_str(&piece);
            tokens_generated += 1;

            // Stream token to callback
            if !on_token(&piece) {
                user_stopped = true;
                break;
            }

            if let Some((matched_seq, pos)) = self.find_stop_sequence(&generated_text, &stop_sequences) {
                stop_reason = StopReason::StopSequence;
                generated_text.truncate(pos);
                break;
            }
        }

        let elapsed = start_time.elapsed();
        let tokens_per_second = if elapsed.as_secs_f32() > 0.0 {
            tokens_generated as f32 / elapsed.as_secs_f32()
        } else {
            0.0
        };

        Ok(GenerationResult {
            text: generated_text,
            tokens_generated,
            time_ms: elapsed.as_millis(),
            tokens_per_second,
            stop_reason: if user_stopped { StopReason::MaxTokens } else { stop_reason },
        })
    }

    /// Update generation configuration
    pub async fn update_config(&self, config: GGUFInferenceConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> GGUFInferenceConfig {
        self.config.read().await.clone()
    }

    /// Sample next token with temperature, top-k, top-p
    fn sample_token(&self, logits: &Tensor, config: &GGUFInferenceConfig) -> Result<u32> {
        let logits = logits.to_vec1::<f32>()?;

        // Apply temperature
        let logits: Vec<f32> = if config.temperature > 0.0 {
            logits.iter().map(|&l| l / config.temperature).collect()
        } else {
            logits
        };

        // Apply top-k filtering
        let mut logits_with_idx: Vec<(usize, f32)> = logits.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        logits_with_idx.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if config.top_k > 0 {
            logits_with_idx.truncate(config.top_k as usize);
        }

        // Apply top-p (nucleus) filtering
        if config.top_p < 1.0 {
            let total: f32 = logits_with_idx.iter().map(|(_, v)| v.exp()).sum();
            let mut cumsum = 0.0;
            let mut cutoff = logits_with_idx.len();

            for (i, (_, logit)) in logits_with_idx.iter().enumerate() {
                cumsum += logit.exp() / total;
                if cumsum > config.top_p {
                    cutoff = i + 1;
                    break;
                }
            }
            logits_with_idx.truncate(cutoff);
        }

        // Sample from remaining tokens
        use rand::Rng;
        let mut rng = rand::thread_rng();

        if logits_with_idx.is_empty() {
            return Err(anyhow!("No valid tokens to sample from"));
        }

        let weights: Vec<f32> = logits_with_idx.iter().map(|(_, v)| v.exp()).collect();
        let total_weight: f32 = weights.iter().sum();
        let mut r = rng.gen::<f32>() * total_weight;

        for (i, &w) in weights.iter().enumerate() {
            r -= w;
            if r <= 0.0 {
                return Ok(logits_with_idx[i].0 as u32);
            }
        }

        Ok(logits_with_idx[0].0 as u32)
    }

    /// Find stop sequence in generated text
    fn find_stop_sequence(&self, text: &str, stop_sequences: &[String]) -> Option<(String, usize)> {
        let mut best_match: Option<(String, usize)> = None;

        for seq in stop_sequences {
            if text.ends_with(seq) {
                if let Some(pos) = text.rfind(seq) {
                    if pos + seq.len() == text.len() {
                        match &best_match {
                            None => best_match = Some((seq.clone(), pos)),
                            Some((prev_seq, _)) => {
                                if seq.len() > prev_seq.len() {
                                    best_match = Some((seq.clone(), pos));
                                }
                            }
                        }
                    }
                }
            }
        }

        best_match
    }

    /// Create a fallback tokenizer when tokenizer.json is not available
    fn create_fallback_tokenizer(&self) -> Result<Tokenizer> {
        // PRODUCTION: Fail loudly instead of using broken tokenizer
        Err(anyhow!(
            "Tokenizer not found. GGUF models require a tokenizer.json file.\n\
             \n\
             Solutions:\n\
             1. Place tokenizer.json in the same directory as your model file\n\
             2. Download from HuggingFace model repo (usually named 'tokenizer.json')\n\
             3. Use a model that includes tokenizer in the download\n\
             \n\
             Example: For TheBloke models, download both:\n\
             - model.gguf (the model weights)\n\
             - tokenizer.json (from the original model repo)"
        ))
    }

    /// Get model information
    pub async fn get_model_info(&self) -> Result<serde_json::Value> {
        let model_lock = self.model.read().await;
        if model_lock.is_none() {
            return Err(anyhow!("No model loaded"));
        }

        let config = self.config.read().await;

        Ok(serde_json::json!({
            "model_path": config.model_path,
            "n_ctx": config.n_ctx,
            "n_threads": config.n_threads,
            "device": format!("{:?}", self.device),
            "backend": "Candle (Pure Rust)",
        }))
    }
}

// Thread safety
unsafe impl Send for GGUFInferenceEngine {}
unsafe impl Sync for GGUFInferenceEngine {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = GGUFInferenceEngine::new();
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_model_not_loaded() {
        let engine = GGUFInferenceEngine::new().unwrap();
        assert!(!engine.is_model_loaded().await);

        let result = engine.generate("Hello", 10, vec![]).await;
        assert!(result.is_err());
    }
}
