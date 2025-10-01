/// Shared utility functions for BEAR AI
///
/// This module contains reusable functions that are used across multiple
/// components to avoid code duplication and ensure consistency.

/// Calculate cosine similarity between two vectors
///
/// Used for comparing embedding vectors in RAG and other ML operations.
/// Returns a value between -1.0 and 1.0, where:
/// - 1.0 = identical direction (perfect similarity)
/// - 0.0 = orthogonal (no similarity)
/// - -1.0 = opposite direction (perfect dissimilarity)
///
/// # Arguments
/// * `vec1` - First vector
/// * `vec2` - Second vector (must have same length as vec1)
///
/// # Returns
/// Cosine similarity score, or 0.0 if vectors have different lengths or zero magnitude
///
/// # Example
/// ```rust
/// let vec1 = vec![1.0, 2.0, 3.0];
/// let vec2 = vec![2.0, 4.0, 6.0];
/// let similarity = cosine_similarity(&vec1, &vec2);
/// assert!((similarity - 1.0).abs() < 0.001); // Nearly identical direction
/// ```
pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    // Validate input vectors
    if vec1.len() != vec2.len() {
        tracing::warn!(
            "Cosine similarity: vector length mismatch ({} vs {})",
            vec1.len(),
            vec2.len()
        );
        return 0.0;
    }

    if vec1.is_empty() || vec2.is_empty() {
        return 0.0;
    }

    // Calculate dot product
    let dot_product: f32 = vec1.iter()
        .zip(vec2.iter())
        .map(|(a, b)| a * b)
        .sum();

    // Calculate magnitudes
    let norm1: f32 = vec1.iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt();

    let norm2: f32 = vec2.iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt();

    // Avoid division by zero
    if norm1 == 0.0 || norm2 == 0.0 {
        tracing::warn!("Cosine similarity: zero magnitude vector detected");
        return 0.0;
    }

    // Return cosine similarity
    dot_product / (norm1 * norm2)
}

/// Estimate model size in MB based on parameters and quantization
///
/// Provides a reasonable estimate for model storage requirements.
/// Used when actual file size is not available from model registry.
///
/// # Arguments
/// * `params_billions` - Model parameters in billions (e.g., 7.0 for 7B model)
/// * `quantization` - Quantization format (e.g., "Q4_K_M", "FP16", "FP32")
///
/// # Returns
/// Estimated size in megabytes
///
/// # Quantization Sizes
/// - Q2_K: ~2 bits per parameter
/// - Q4_K_M: ~4.5 bits per parameter (most common for GGUF)
/// - Q5_K_M: ~5.5 bits per parameter
/// - Q6_K: ~6.5 bits per parameter
/// - Q8_0: ~8.5 bits per parameter
/// - FP16: 16 bits per parameter
/// - FP32: 32 bits per parameter
pub fn estimate_model_size_mb(params_billions: f32, quantization: &str) -> u64 {
    // Bits per parameter based on quantization
    let bits_per_param = match quantization.to_uppercase().as_str() {
        // GGUF quantization formats
        q if q.starts_with("Q2") => 2.0,
        q if q.starts_with("Q3") => 3.5,
        q if q.starts_with("Q4") => 4.5,
        q if q.starts_with("Q5") => 5.5,
        q if q.starts_with("Q6") => 6.5,
        q if q.starts_with("Q8") => 8.5,

        // Float precision formats
        "FP16" | "FLOAT16" | "HALF" => 16.0,
        "FP32" | "FLOAT32" | "FULL" => 32.0,
        "BF16" | "BFLOAT16" => 16.0,

        // Default to Q4_K_M (most common)
        _ => {
            tracing::warn!(
                "Unknown quantization '{}', assuming Q4_K_M",
                quantization
            );
            4.5
        }
    };

    // Calculate size
    // params_billions * 1e9 * bits_per_param / 8 / (1024 * 1024)
    let size_mb = (params_billions * 1_000_000_000.0 * bits_per_param) / (8.0 * 1024.0 * 1024.0);

    // Add ~10% overhead for model metadata, vocabulary, etc.
    let size_with_overhead = size_mb * 1.1;

    size_with_overhead as u64
}

/// Parse model size from model ID string
///
/// Extracts parameter count from common model naming patterns.
///
/// # Examples
/// - "llama-2-7b" → Some(7.0)
/// - "mistral-7b-instruct" → Some(7.0)
/// - "tinyllama-1.1b" → Some(1.1)
/// - "phi-2" → Some(2.7) (known model)
/// - "unknown-model" → None
pub fn parse_model_params_from_id(model_id: &str) -> Option<f32> {
    let id_lower = model_id.to_lowercase();

    // Known model sizes
    if id_lower.contains("phi-2") || id_lower.contains("phi2") {
        return Some(2.7); // Phi-2 is 2.7B parameters
    }

    if id_lower.contains("tinyllama") {
        return Some(1.1); // TinyLlama is 1.1B
    }

    // Extract from patterns like "7b", "13b", "70b", "1.1b"
    let re = regex::Regex::new(r"(\d+\.?\d*)\s*b(?:illion)?").ok()?;

    if let Some(caps) = re.captures(&id_lower) {
        if let Some(num_str) = caps.get(1) {
            return num_str.as_str().parse::<f32>().ok();
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        // Identical vectors
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!((sim - 1.0).abs() < 0.001);

        // Orthogonal vectors
        let vec1 = vec![1.0, 0.0];
        let vec2 = vec![0.0, 1.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!(sim.abs() < 0.001);

        // Opposite vectors
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![-1.0, -2.0, -3.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert!((sim + 1.0).abs() < 0.001);

        // Length mismatch
        let vec1 = vec![1.0, 2.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&vec1, &vec2);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_model_size_estimation() {
        // 7B Q4_K_M model
        let size = estimate_model_size_mb(7.0, "Q4_K_M");
        assert!(size >= 3500 && size <= 4500); // ~4GB

        // 7B FP16 model
        let size = estimate_model_size_mb(7.0, "FP16");
        assert!(size >= 13000 && size <= 15000); // ~14GB

        // 1.1B Q4_K_M model (TinyLlama)
        let size = estimate_model_size_mb(1.1, "Q4_K_M");
        assert!(size >= 500 && size <= 800); // ~600MB
    }

    #[test]
    fn test_parse_model_params() {
        assert_eq!(parse_model_params_from_id("llama-2-7b"), Some(7.0));
        assert_eq!(parse_model_params_from_id("mistral-7b-instruct"), Some(7.0));
        assert_eq!(parse_model_params_from_id("llama-2-13b"), Some(13.0));
        assert_eq!(parse_model_params_from_id("llama-2-70b"), Some(70.0));
        assert_eq!(parse_model_params_from_id("tinyllama-1.1b"), Some(1.1));
        assert_eq!(parse_model_params_from_id("phi-2"), Some(2.7));
        assert_eq!(parse_model_params_from_id("gpt-3.5"), None);
    }
}
