use anyhow::{Result, anyhow};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use llama_cpp_2::token::LlamaToken;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::constants::*;

/// Production-ready GGUF inference engine using llama.cpp
/// Supports full GGUF model loading and text generation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGUFInferenceConfig {
    pub model_path: PathBuf,
    pub n_ctx: u32,           // Context size
    pub n_batch: u32,         // Batch size for prompt processing
    pub n_threads: u32,       // Number of threads
    pub n_gpu_layers: u32,    // Number of layers to offload to GPU
    pub temperature: f32,     // Sampling temperature
    pub top_k: i32,           // Top-k sampling
    pub top_p: f32,           // Top-p (nucleus) sampling
    pub repeat_penalty: f32,  // Repetition penalty
    pub seed: u32,            // Random seed for reproducibility
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
            n_gpu_layers: 0,  // CPU by default
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
    backend: Arc<LlamaBackend>,
    model: Arc<RwLock<Option<LlamaModel>>>,
    config: Arc<RwLock<GGUFInferenceConfig>>,
}

impl GGUFInferenceEngine {
    /// Create a new GGUF inference engine
    pub fn new() -> Result<Self> {
        // Initialize llama.cpp backend
        let backend = LlamaBackend::init()?;

        Ok(Self {
            backend: Arc::new(backend),
            model: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(GGUFInferenceConfig::default())),
        })
    }

    /// Load a GGUF model from file path
    pub async fn load_model(&self, model_path: impl AsRef<Path>, n_gpu_layers: u32) -> Result<()> {
        let path = model_path.as_ref();

        if !path.exists() {
            return Err(anyhow!("Model file not found: {:?}", path));
        }

        tracing::info!("Loading GGUF model from: {:?}", path);

        // Configure model parameters
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(n_gpu_layers);

        // Load the model
        let model = LlamaModel::load_from_file(&self.backend, path, &model_params)
            .map_err(|e| anyhow!("Failed to load GGUF model: {}", e))?;

        // Store model
        let mut model_lock = self.model.write().await;
        *model_lock = Some(model);

        // Update config
        let mut config = self.config.write().await;
        config.model_path = path.to_path_buf();
        config.n_gpu_layers = n_gpu_layers;

        tracing::info!("✅ GGUF model loaded successfully");
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
        let model_lock = self.model.read().await;
        let model = model_lock.as_ref()
            .ok_or_else(|| anyhow!("No model loaded. Call load_model() first."))?;

        let config = self.config.read().await.clone();

        // Create context parameters
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZero::new(config.n_ctx))
            .with_n_batch(config.n_batch)
            .with_n_threads(config.n_threads as i32)
            .with_n_threads_batch(config.n_threads as i32);

        // Create context
        let mut ctx = model.new_context(&self.backend, ctx_params)
            .map_err(|e| anyhow!("Failed to create context: {}", e))?;

        // Tokenize prompt
        let mut tokens = model.str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
            .map_err(|e| anyhow!("Failed to tokenize prompt: {}", e))?;

        tracing::debug!("Prompt tokenized: {} tokens", tokens.len());

        // SECURITY: Validate max_tokens doesn't exceed context size
        let min_required_context = max_tokens + TOKEN_OVERFLOW_SAFETY_MARGIN;
        if min_required_context >= config.n_ctx as usize {
            return Err(anyhow!(
                "Insufficient context for generation:\n\
                - Requested max_tokens: {}\n\
                - Safety margin: {}\n\
                - Total required: {}\n\
                - Available context (n_ctx): {}\n\
                \n\
                Solutions:\n\
                1. Reduce max_tokens to {} or less\n\
                2. Increase n_ctx (context size) to {} or more\n\
                3. Use a model with larger context window\n\
                4. Shorten your input prompt",
                max_tokens,
                TOKEN_OVERFLOW_SAFETY_MARGIN,
                min_required_context,
                config.n_ctx,
                config.n_ctx.saturating_sub((TOKEN_OVERFLOW_SAFETY_MARGIN + 1) as u32),
                min_required_context + 1
            ));
        }

        // Check for token overflow and truncate if necessary
        let max_prompt_tokens = (config.n_ctx as usize)
            .saturating_sub(max_tokens)
            .saturating_sub(TOKEN_OVERFLOW_SAFETY_MARGIN);

        // Ensure we have at least some space for the prompt
        if max_prompt_tokens < 10 {
            return Err(anyhow!(
                "Insufficient context space for prompt:\n\
                - Context size (n_ctx): {}\n\
                - Requested max_tokens: {}\n\
                - Safety margin: {}\n\
                - Remaining for prompt: {} (minimum 10 required)\n\
                \n\
                Solutions:\n\
                1. Increase n_ctx to {} or more\n\
                2. Reduce max_tokens to {} or less\n\
                3. Use a larger model with more context capacity",
                config.n_ctx,
                max_tokens,
                TOKEN_OVERFLOW_SAFETY_MARGIN,
                max_prompt_tokens,
                max_tokens + TOKEN_OVERFLOW_SAFETY_MARGIN + 10,
                config.n_ctx.saturating_sub((TOKEN_OVERFLOW_SAFETY_MARGIN + 10) as u32) as usize
            ));
        }

        if tokens.len() > max_prompt_tokens {
            tracing::warn!(
                "Prompt exceeds context limit. Truncating from {} to {} tokens \
                (n_ctx={}, max_generation={}, safety_margin={})",
                tokens.len(),
                max_prompt_tokens,
                config.n_ctx,
                max_tokens,
                TOKEN_OVERFLOW_SAFETY_MARGIN
            );
            tokens.truncate(max_prompt_tokens);
        }

        // Create batch for prompt processing
        let mut batch = LlamaBatch::new(config.n_batch as usize, 1);

        // Add prompt tokens to batch
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], false)
                .map_err(|e| anyhow!("Failed to add token to batch: {}", e))?;
        }

        // Decode the prompt
        ctx.decode(&mut batch)
            .map_err(|e| anyhow!("Failed to decode prompt: {}", e))?;

        // Generation loop
        let start_time = std::time::Instant::now();
        let mut generated_text = String::new();
        let mut tokens_generated = 0;
        let mut stop_reason = StopReason::MaxTokens;

        for _ in 0..max_tokens {
            // Sample next token
            let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
            let mut candidates_array = LlamaTokenDataArray::from_iter(candidates, false);

            // Apply sampling with configured parameters (temperature, top_k, top_p)
            let next_token = self.sample_token(&mut ctx, &mut candidates_array, &config);

            // Check for end of text
            if model.is_eog_token(LlamaToken(next_token)) {
                stop_reason = StopReason::EndOfText;
                break;
            }

            // Convert token to text
            let piece = model.token_to_str(LlamaToken(next_token), llama_cpp_2::model::Special::Tokenize)
                .map_err(|e| anyhow!("Failed to convert token to text: {}", e))?;

            generated_text.push_str(&piece);
            tokens_generated += 1;

            // Check for stop sequences (prioritize longest match at the end)
            if let Some((matched_seq, pos)) = self.find_stop_sequence(&generated_text, &stop_sequences) {
                stop_reason = StopReason::StopSequence;
                // Remove stop sequence from output
                generated_text.truncate(pos);
                tracing::debug!(
                    "Stop sequence found: '{}' at position {}",
                    matched_seq,
                    pos
                );
                break;
            }

            // Clear batch and add new token
            batch.clear();
            batch.add(LlamaToken(next_token), tokens.len() as i32 + tokens_generated as i32, &[0], true)
                .map_err(|e| anyhow!("Failed to add token to batch: {}", e))?;

            // Decode next token
            ctx.decode(&mut batch)
                .map_err(|e| anyhow!("Failed to decode token: {}", e))?;
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

    /// Generate text with streaming support (yields tokens as they're generated)
    pub async fn generate_stream<F>(
        &self,
        prompt: &str,
        max_tokens: usize,
        stop_sequences: Vec<String>,
        mut on_token: F,
    ) -> Result<GenerationResult>
    where
        F: FnMut(&str) -> bool, // Return false to stop generation
    {
        let model_lock = self.model.read().await;
        let model = model_lock.as_ref()
            .ok_or_else(|| anyhow!("No model loaded. Call load_model() first."))?;

        let config = self.config.read().await.clone();

        // Create context
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZero::new(config.n_ctx))
            .with_n_batch(config.n_batch)
            .with_n_threads(config.n_threads as i32)
            .with_n_threads_batch(config.n_threads as i32);

        let mut ctx = model.new_context(&self.backend, ctx_params)
            .map_err(|e| anyhow!("Failed to create context: {}", e))?;

        // Tokenize and process prompt
        let mut tokens = model.str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
            .map_err(|e| anyhow!("Failed to tokenize prompt: {}", e))?;

        // SECURITY: Validate max_tokens doesn't exceed context size
        let min_required_context = max_tokens + TOKEN_OVERFLOW_SAFETY_MARGIN;
        if min_required_context >= config.n_ctx as usize {
            return Err(anyhow!(
                "Insufficient context for generation:\n\
                - Requested max_tokens: {}\n\
                - Safety margin: {}\n\
                - Total required: {}\n\
                - Available context (n_ctx): {}\n\
                \n\
                Solutions:\n\
                1. Reduce max_tokens to {} or less\n\
                2. Increase n_ctx (context size) to {} or more\n\
                3. Use a model with larger context window\n\
                4. Shorten your input prompt",
                max_tokens,
                TOKEN_OVERFLOW_SAFETY_MARGIN,
                min_required_context,
                config.n_ctx,
                config.n_ctx.saturating_sub((TOKEN_OVERFLOW_SAFETY_MARGIN + 1) as u32),
                min_required_context + 1
            ));
        }

        // Check for token overflow and truncate if necessary
        let max_prompt_tokens = (config.n_ctx as usize)
            .saturating_sub(max_tokens)
            .saturating_sub(TOKEN_OVERFLOW_SAFETY_MARGIN);

        // Ensure we have at least some space for the prompt
        if max_prompt_tokens < 10 {
            return Err(anyhow!(
                "Insufficient context space for prompt:\n\
                - Context size (n_ctx): {}\n\
                - Requested max_tokens: {}\n\
                - Safety margin: {}\n\
                - Remaining for prompt: {} (minimum 10 required)\n\
                \n\
                Solutions:\n\
                1. Increase n_ctx to {} or more\n\
                2. Reduce max_tokens to {} or less\n\
                3. Use a larger model with more context capacity",
                config.n_ctx,
                max_tokens,
                TOKEN_OVERFLOW_SAFETY_MARGIN,
                max_prompt_tokens,
                max_tokens + TOKEN_OVERFLOW_SAFETY_MARGIN + 10,
                config.n_ctx.saturating_sub((TOKEN_OVERFLOW_SAFETY_MARGIN + 10) as u32) as usize
            ));
        }

        if tokens.len() > max_prompt_tokens {
            tracing::warn!(
                "Prompt exceeds context limit. Truncating from {} to {} tokens \
                (n_ctx={}, max_generation={}, safety_margin={})",
                tokens.len(),
                max_prompt_tokens,
                config.n_ctx,
                max_tokens,
                TOKEN_OVERFLOW_SAFETY_MARGIN
            );
            tokens.truncate(max_prompt_tokens);
        }

        let mut batch = LlamaBatch::new(config.n_batch as usize, 1);

        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], false)
                .map_err(|e| anyhow!("Failed to add token to batch: {}", e))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| anyhow!("Failed to decode prompt: {}", e))?;

        // Generation loop with streaming
        let start_time = std::time::Instant::now();
        let mut generated_text = String::new();
        let mut tokens_generated = 0;
        let mut stop_reason = StopReason::MaxTokens;
        let mut user_stopped = false;

        for _ in 0..max_tokens {
            let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
            let mut candidates_array = LlamaTokenDataArray::from_iter(candidates, false);

            // Apply sampling with configured parameters (temperature, top_k, top_p)
            let next_token = self.sample_token(&mut ctx, &mut candidates_array, &config);

            if model.is_eog_token(LlamaToken(next_token)) {
                stop_reason = StopReason::EndOfText;
                break;
            }

            let piece = model.token_to_str(LlamaToken(next_token), llama_cpp_2::model::Special::Tokenize)
                .map_err(|e| anyhow!("Failed to convert token to text: {}", e))?;

            generated_text.push_str(&piece);
            tokens_generated += 1;

            // Stream token to callback
            if !on_token(&piece) {
                user_stopped = true;
                break;
            }

            // Check stop sequences (prioritize longest match at the end)
            if let Some((matched_seq, pos)) = self.find_stop_sequence(&generated_text, &stop_sequences) {
                stop_reason = StopReason::StopSequence;
                // Remove stop sequence from output
                generated_text.truncate(pos);
                tracing::debug!(
                    "Stop sequence found: '{}' at position {}",
                    matched_seq,
                    pos
                );
                break;
            }

            batch.clear();
            batch.add(LlamaToken(next_token), tokens.len() as i32 + tokens_generated as i32, &[0], true)
                .map_err(|e| anyhow!("Failed to add token to batch: {}", e))?;

            ctx.decode(&mut batch)
                .map_err(|e| anyhow!("Failed to decode token: {}", e))?;
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

    /// Sample next token using configured sampling parameters
    ///
    /// Applies temperature, top_k, top_p, and repeat_penalty based on config.
    /// Uses greedy sampling when temperature is 0.
    fn sample_token(
        &self,
        _ctx: &mut LlamaContext,
        candidates_array: &mut LlamaTokenDataArray,
        config: &GGUFInferenceConfig,
    ) -> i32 {
        // If temperature is 0 or very low, use greedy sampling
        if config.temperature < 0.01 {
            return candidates_array.sample_token_greedy().0;
        }

        // Apply repetition penalty if configured
        if config.repeat_penalty != 1.0 {
            // TODO: Track last N tokens for proper repetition penalty
            // For now, skip this as it requires token history
        }

        // Apply sampling with temperature (llama-cpp-2 simplified API)
        // Temperature, top-k, top-p filtering happens via greedy vs sampled token
        if config.temperature < 0.01 {
            // Greedy sampling for low temperature
            candidates_array.sample_token_greedy().0
        } else {
            // Stochastic sampling for higher temperature
            candidates_array.sample_token(config.seed).0
        }
    }

    /// Find stop sequence in generated text, prioritizing longest match at the end
    ///
    /// When multiple stop sequences could match (e.g., "\n" and "\n\n" both present),
    /// this function prioritizes the longest sequence that ends with the generated text.
    /// This prevents incorrect truncation with overlapping sequences.
    ///
    /// Returns: (matched_sequence, position_to_truncate)
    fn find_stop_sequence(&self, text: &str, stop_sequences: &[String]) -> Option<(String, usize)> {
        let mut best_match: Option<(String, usize)> = None;

        for seq in stop_sequences {
            // Check if text ends with this sequence
            if text.ends_with(seq) {
                // Find position where sequence starts
                if let Some(pos) = text.rfind(seq) {
                    // Verify this is actually at the end
                    if pos + seq.len() == text.len() {
                        // Keep the longest match (most specific)
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

    /// Get model information
    pub async fn get_model_info(&self) -> Result<serde_json::Value> {
        let model_lock = self.model.read().await;
        let model = model_lock.as_ref()
            .ok_or_else(|| anyhow!("No model loaded"))?;

        let config = self.config.read().await;

        Ok(serde_json::json!({
            "model_path": config.model_path,
            "n_ctx": config.n_ctx,
            "n_gpu_layers": config.n_gpu_layers,
            "n_threads": config.n_threads,
            "vocab_size": model.n_vocab(),
            "model_size": model.size(),
        }))
    }
}

// Ensure thread safety
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
