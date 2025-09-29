use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tokio::fs;
use hf_hub::{api::tokio::Api, Repo, RepoType};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub model_type: String,
    pub repo_id: String,
    pub model_file: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub context_length: usize,
}

pub struct LLMManager {
    models: HashMap<String, ModelConfig>,
    active_model: Option<String>,
    models_dir: PathBuf,
    generation_config: Arc<RwLock<GenerationConfig>>,
}

#[derive(Debug, Clone)]
struct GenerationConfig {
    temperature: f32,
    max_tokens: usize,
    top_p: f32,
    top_k: usize,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            max_tokens: 1024,
            top_p: 0.95,
            top_k: 40,
        }
    }
}

impl LLMManager {
    pub fn new() -> Self {
        let models_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("legal-ai-assistant")
            .join("models");

        Self {
            models: HashMap::new(),
            active_model: None,
            models_dir,
            generation_config: Arc::new(RwLock::new(GenerationConfig::default())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        fs::create_dir_all(&self.models_dir).await?;
        self.load_available_models().await?;
        Ok(())
    }

    async fn load_available_models(&mut self) -> Result<()> {
        let default_models = vec![
            ModelConfig {
                name: "tinyllama-1.1b".to_string(),
                model_type: "llama".to_string(),
                repo_id: "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF".to_string(),
                model_file: "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf".to_string(),
                max_tokens: 1024,
                temperature: 0.8,
                context_length: 2048,
            },
            ModelConfig {
                name: "phi-2".to_string(),
                model_type: "phi".to_string(),
                repo_id: "TheBloke/phi-2-GGUF".to_string(),
                model_file: "phi-2.Q4_K_M.gguf".to_string(),
                max_tokens: 1024,
                temperature: 0.8,
                context_length: 2048,
            },
            ModelConfig {
                name: "mistral-7b-instruct".to_string(),
                model_type: "mistral".to_string(),
                repo_id: "TheBloke/Mistral-7B-Instruct-v0.2-GGUF".to_string(),
                model_file: "mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string(),
                max_tokens: 2048,
                temperature: 0.8,
                context_length: 4096,
            },
        ];

        for model in default_models {
            self.models.insert(model.name.clone(), model);
        }

        Ok(())
    }

    pub async fn download_model(&mut self, model_name: &str) -> Result<()> {
        let model_config = self.models.get(model_name)
            .ok_or_else(|| anyhow!("Model not found: {}", model_name))?
            .clone();

        println!("Downloading model: {} from {}", model_name, model_config.repo_id);

        let api = Api::new()?;
        let repo = api.repo(Repo::new(model_config.repo_id.clone(), RepoType::Model));

        let model_dir = self.models_dir.join(&model_name);
        fs::create_dir_all(&model_dir).await?;

        println!("Downloading model file: {}", model_config.model_file);
        let model_path = repo.get(&model_config.model_file).await?;
        let dest_model = model_dir.join(&model_config.model_file);

        if dest_model.exists() {
            println!("Model file already exists at {:?}", dest_model);
        } else {
            tokio::fs::copy(&model_path, &dest_model).await?;
            println!("Model {} downloaded successfully to {:?}", model_name, dest_model);
        }

        Ok(())
    }

    pub async fn load_model(&mut self, model_name: &str) -> Result<()> {
        if !self.models.contains_key(model_name) {
            return Err(anyhow!("Model not found: {}", model_name));
        }

        let model_dir = self.models_dir.join(model_name);

        if !model_dir.exists() {
            println!("Model directory doesn't exist, downloading model first...");
            self.download_model(model_name).await?;
        }

        let model_config = self.models.get(model_name).cloned().unwrap();
        let model_file = model_dir.join(&model_config.model_file);

        if !model_file.exists() {
            println!("Model file not found, downloading...");
            self.download_model(model_name).await?;
        }

        self.active_model = Some(model_name.to_string());

        let mut config = self.generation_config.write().await;
        config.temperature = model_config.temperature;
        config.max_tokens = model_config.max_tokens;

        println!("Model {} loaded and ready for inference", model_name);
        Ok(())
    }

    pub async fn generate_response(&mut self, prompt: &str, model_name: &str) -> Result<String> {
        if self.active_model.as_deref() != Some(model_name) {
            self.load_model(model_name).await?;
        }

        let model_config = self.models.get(model_name)
            .ok_or_else(|| anyhow!("Model not found: {}", model_name))?;

        let model_dir = self.models_dir.join(model_name);
        let model_file = model_dir.join(&model_config.model_file);

        if !model_file.exists() {
            return Err(anyhow!(
                "Model file not found. Please ensure the model is downloaded first. \
                Expected path: {:?}",
                model_file
            ));
        }

        let formatted_prompt = match model_config.model_type.as_str() {
            "llama" | "mistral" => {
                format!("<s>[INST] {} [/INST]", prompt)
            }
            "phi" => {
                format!("Instruct: {}\nOutput:", prompt)
            }
            _ => prompt.to_string(),
        };

    println!("Processing prompt with model: {}", model_name);
        println!("Model type: {}", model_config.model_type);
        println!("Model file location: {:?}", model_file);

        let response = format!(
            "Model '{}' loaded from {:?}\n\
            Configuration:\n\
            - Type: {}\n\
            - Temperature: {}\n\
            - Max tokens: {}\n\
            - Context length: {}\n\n\
            To enable actual inference, a compatible GGUF runtime needs to be integrated.\n\
            The model files are ready at: {:?}\n\n\
            Your prompt: {}",
            model_name,
            model_file,
            model_config.model_type,
            model_config.temperature,
            model_config.max_tokens,
            model_config.context_length,
            model_dir,
            formatted_prompt
        );

        Ok(response)
    }

    pub async fn list_models(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }

    pub async fn get_model_info(&self, model_name: &str) -> Option<ModelConfig> {
        self.models.get(model_name).cloned()
    }

    pub async fn unload_model(&mut self) -> Result<()> {
        self.active_model = None;
        Ok(())
    }

    pub fn get_active_model(&self) -> Option<String> {
        self.active_model.clone()
    }
}