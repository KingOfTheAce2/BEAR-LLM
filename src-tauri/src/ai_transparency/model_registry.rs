use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMapping {
    pub pattern: String,
    pub huggingface_id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    mappings: Vec<ModelMapping>,
    custom_mappings: HashMap<String, String>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry {
    pub fn new() -> Self {
        let default_mappings = vec![
            // Llama models
            ModelMapping {
                pattern: r"^llama-2-7b-chat".to_string(),
                huggingface_id: "meta-llama/Llama-2-7b-chat-hf".to_string(),
                description: "Llama 2 7B Chat".to_string(),
            },
            ModelMapping {
                pattern: r"^llama-2-13b-chat".to_string(),
                huggingface_id: "meta-llama/Llama-2-13b-chat-hf".to_string(),
                description: "Llama 2 13B Chat".to_string(),
            },
            ModelMapping {
                pattern: r"^llama-2-70b-chat".to_string(),
                huggingface_id: "meta-llama/Llama-2-70b-chat-hf".to_string(),
                description: "Llama 2 70B Chat".to_string(),
            },
            ModelMapping {
                pattern: r"^llama-3-8b-instruct".to_string(),
                huggingface_id: "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
                description: "Llama 3 8B Instruct".to_string(),
            },
            ModelMapping {
                pattern: r"^llama-3-70b-instruct".to_string(),
                huggingface_id: "meta-llama/Meta-Llama-3-70B-Instruct".to_string(),
                description: "Llama 3 70B Instruct".to_string(),
            },
            // Mistral models
            ModelMapping {
                pattern: r"^mistral-7b-instruct".to_string(),
                huggingface_id: "mistralai/Mistral-7B-Instruct-v0.2".to_string(),
                description: "Mistral 7B Instruct".to_string(),
            },
            ModelMapping {
                pattern: r"^mixtral-8x7b-instruct".to_string(),
                huggingface_id: "mistralai/Mixtral-8x7B-Instruct-v0.1".to_string(),
                description: "Mixtral 8x7B Instruct".to_string(),
            },
            // Microsoft Phi models
            ModelMapping {
                pattern: r"^phi-2".to_string(),
                huggingface_id: "microsoft/phi-2".to_string(),
                description: "Microsoft Phi-2".to_string(),
            },
            ModelMapping {
                pattern: r"^phi-3-mini".to_string(),
                huggingface_id: "microsoft/Phi-3-mini-4k-instruct".to_string(),
                description: "Microsoft Phi-3 Mini".to_string(),
            },
            // Google Gemma models
            ModelMapping {
                pattern: r"^gemma-2b".to_string(),
                huggingface_id: "google/gemma-2b".to_string(),
                description: "Google Gemma 2B".to_string(),
            },
            ModelMapping {
                pattern: r"^gemma-7b".to_string(),
                huggingface_id: "google/gemma-7b".to_string(),
                description: "Google Gemma 7B".to_string(),
            },
            // OpenHermes models
            ModelMapping {
                pattern: r"^openhermes".to_string(),
                huggingface_id: "teknium/OpenHermes-2.5-Mistral-7B".to_string(),
                description: "OpenHermes 2.5".to_string(),
            },
            // Zephyr models
            ModelMapping {
                pattern: r"^zephyr-7b".to_string(),
                huggingface_id: "HuggingFaceH4/zephyr-7b-beta".to_string(),
                description: "Zephyr 7B".to_string(),
            },
            // Nous Hermes models
            ModelMapping {
                pattern: r"^nous-hermes".to_string(),
                huggingface_id: "NousResearch/Nous-Hermes-2-Mixtral-8x7B-DPO".to_string(),
                description: "Nous Hermes 2".to_string(),
            },
            // Orca models
            ModelMapping {
                pattern: r"^orca-2".to_string(),
                huggingface_id: "microsoft/Orca-2-7b".to_string(),
                description: "Microsoft Orca 2".to_string(),
            },
            // Starling models
            ModelMapping {
                pattern: r"^starling-lm".to_string(),
                huggingface_id: "berkeley-nest/Starling-LM-7B-alpha".to_string(),
                description: "Starling LM".to_string(),
            },
            // Vicuna models
            ModelMapping {
                pattern: r"^vicuna".to_string(),
                huggingface_id: "lmsys/vicuna-7b-v1.5".to_string(),
                description: "Vicuna".to_string(),
            },
        ];

        Self {
            mappings: default_mappings,
            custom_mappings: HashMap::new(),
        }
    }

    /// Load custom mappings from config file
    pub fn load_from_file(config_path: PathBuf) -> Result<Self, String> {
        let mut registry = Self::new();

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config: {}", e))?;

            let custom_mappings: HashMap<String, String> = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse config: {}", e))?;

            registry.custom_mappings = custom_mappings;
        }

        Ok(registry)
    }

    /// Save custom mappings to config file
    pub fn save_to_file(&self, config_path: PathBuf) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.custom_mappings)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, json).map_err(|e| format!("Failed to write config: {}", e))
    }

    /// Resolve GGUF filename to HuggingFace model ID
    pub fn resolve_model_id(&self, filename: &str) -> Option<String> {
        // Remove file extension and path
        let clean_name = filename
            .trim_end_matches(".gguf")
            .trim_end_matches(".bin")
            .to_lowercase()
            .replace('_', "-");

        // Check custom mappings first (exact match)
        if let Some(id) = self.custom_mappings.get(&clean_name) {
            return Some(id.clone());
        }

        // Try pattern matching
        for mapping in &self.mappings {
            if let Ok(re) = Regex::new(&mapping.pattern) {
                if re.is_match(&clean_name) {
                    return Some(mapping.huggingface_id.clone());
                }
            }
        }

        // No match found
        None
    }

    /// Add custom mapping
    pub fn add_custom_mapping(&mut self, filename: String, model_id: String) {
        let clean_name = filename
            .trim_end_matches(".gguf")
            .trim_end_matches(".bin")
            .to_lowercase()
            .replace('_', "-");

        self.custom_mappings.insert(clean_name, model_id);
    }

    /// Remove custom mapping
    pub fn remove_custom_mapping(&mut self, filename: &str) -> bool {
        let clean_name = filename
            .trim_end_matches(".gguf")
            .trim_end_matches(".bin")
            .to_lowercase()
            .replace('_', "-");

        self.custom_mappings.remove(&clean_name).is_some()
    }

    /// Get all mappings (built-in + custom)
    pub fn get_all_mappings(&self) -> Vec<(String, String)> {
        let mut all_mappings: Vec<(String, String)> = self
            .mappings
            .iter()
            .map(|m| (m.pattern.clone(), m.huggingface_id.clone()))
            .collect();

        all_mappings.extend(
            self.custom_mappings
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );

        all_mappings
    }

    /// Extract model name from filename for display
    pub fn extract_model_name(&self, filename: &str) -> String {
        // Remove common suffixes and quantization info
        let name = filename.trim_end_matches(".gguf").trim_end_matches(".bin");

        // Remove quantization suffix (e.g., Q4_K_M, Q5_K_S)
        let re = Regex::new(r"\.(Q\d+_[KM](_[MS])?)$").unwrap();
        let cleaned = re.replace(name, "").to_string();

        // Convert to title case
        cleaned
            .replace('_', " ")
            .replace('-', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_llama_models() {
        let registry = ModelRegistry::new();

        assert_eq!(
            registry.resolve_model_id("llama-2-7b-chat.Q4_K_M.gguf"),
            Some("meta-llama/Llama-2-7b-chat-hf".to_string())
        );

        assert_eq!(
            registry.resolve_model_id("llama-3-8b-instruct.Q5_K_S.gguf"),
            Some("meta-llama/Meta-Llama-3-8B-Instruct".to_string())
        );
    }

    #[test]
    fn test_resolve_mistral_models() {
        let registry = ModelRegistry::new();

        assert_eq!(
            registry.resolve_model_id("mistral-7b-instruct-v0.2.Q4_K_M.gguf"),
            Some("mistralai/Mistral-7B-Instruct-v0.2".to_string())
        );
    }

    #[test]
    fn test_custom_mappings() {
        let mut registry = ModelRegistry::new();

        registry.add_custom_mapping(
            "my-custom-model.gguf".to_string(),
            "user/custom-model".to_string(),
        );

        assert_eq!(
            registry.resolve_model_id("my-custom-model.gguf"),
            Some("user/custom-model".to_string())
        );
    }

    #[test]
    fn test_extract_model_name() {
        let registry = ModelRegistry::new();

        assert_eq!(
            registry.extract_model_name("llama-2-7b-chat.Q4_K_M.gguf"),
            "Llama 2 7b Chat"
        );

        assert_eq!(
            registry.extract_model_name("mistral_7b_instruct.gguf"),
            "Mistral 7b Instruct"
        );
    }

    #[test]
    fn test_unknown_model() {
        let registry = ModelRegistry::new();

        assert_eq!(registry.resolve_model_id("unknown-model.gguf"), None);
    }
}
