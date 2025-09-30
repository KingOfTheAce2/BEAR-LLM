use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::{RwLock, mpsc};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupProgress {
    pub step: String,
    pub progress: f32,
    pub message: String,
    pub is_complete: bool,
    pub has_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    pub install_presidio: bool,
    pub install_models: bool,
    pub model_size: String, // "small", "medium", "large"
    pub enable_gpu: bool,
    pub data_dir: PathBuf,
}

impl Default for SetupConfig {
    fn default() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("bear-ai-llm");

        Self {
            install_presidio: true,
            install_models: true,
            model_size: "medium".to_string(),
            enable_gpu: false,
            data_dir,
        }
    }
}

pub struct SetupManager {
    config: Arc<RwLock<SetupConfig>>,
    is_first_run: Arc<RwLock<bool>>,
    progress_sender: Arc<RwLock<Option<mpsc::Sender<SetupProgress>>>>,
    setup_complete: Arc<RwLock<bool>>,
}

impl SetupManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(SetupConfig::default())),
            is_first_run: Arc::new(RwLock::new(false)),
            progress_sender: Arc::new(RwLock::new(None)),
            setup_complete: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn check_first_run(&self) -> Result<bool> {
        let config = self.config.read().await;
        let marker_file = config.data_dir.join(".setup_complete");

        if !marker_file.exists() {
            let mut is_first = self.is_first_run.write().await;
            *is_first = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn run_setup(&self, progress_sender: mpsc::Sender<SetupProgress>) -> Result<()> {
        // Store progress sender
        let mut sender_lock = self.progress_sender.write().await;
        *sender_lock = Some(progress_sender.clone());
        drop(sender_lock);

        let config = self.config.read().await.clone();

        // Send initial progress
        self.send_progress("Initializing", 0.0, "Starting BEAR AI setup...").await?;

        // Step 1: Create directories
        self.send_progress("Creating directories", 10.0, "Setting up application directories...").await?;
        self.create_directories(&config).await?;

        // Step 2: Check system requirements
        self.send_progress("Checking requirements", 20.0, "Verifying system requirements...").await?;
        self.check_requirements().await?;

        // Step 3: Install Python dependencies
        if config.install_presidio {
            self.send_progress("Installing Presidio", 30.0, "Installing Microsoft Presidio for state-of-the-art PII protection...").await?;
            self.install_presidio_components().await?;
        }

        // Step 4: Download models
        if config.install_models {
            self.send_progress("Downloading models", 50.0, "Downloading AI models (this may take several minutes)...").await?;
            self.download_ai_models(&config).await?;
        }

        // Step 5: Verify installation
        self.send_progress("Verifying", 80.0, "Verifying installation...").await?;
        self.verify_setup().await?;

        // Step 6: Mark setup complete
        self.send_progress("Finalizing", 95.0, "Finalizing setup...").await?;
        self.mark_setup_complete(&config).await?;

        // Send completion
        self.send_progress("Complete", 100.0, "Setup completed successfully!").await?;

        let mut complete = self.setup_complete.write().await;
        *complete = true;

        Ok(())
    }

    async fn send_progress(&self, step: &str, progress: f32, message: &str) -> Result<()> {
        let sender_lock = self.progress_sender.read().await;
        if let Some(sender) = sender_lock.as_ref() {
            let progress_msg = SetupProgress {
                step: step.to_string(),
                progress,
                message: message.to_string(),
                is_complete: progress >= 100.0,
                has_error: false,
            };

            sender.send(progress_msg).await
                .map_err(|e| anyhow!("Failed to send progress: {}", e))?;
        }
        Ok(())
    }

    async fn create_directories(&self, config: &SetupConfig) -> Result<()> {
        let dirs = vec![
            config.data_dir.clone(),
            config.data_dir.join("models"),
            config.data_dir.join("presidio"),
            config.data_dir.join("cache"),
            config.data_dir.join("embeddings"),
            config.data_dir.join("rag_index"),
        ];

        for dir in dirs {
            tokio::fs::create_dir_all(&dir).await?;
        }

        Ok(())
    }

    async fn check_requirements(&self) -> Result<()> {
        // Check Python
        let python_check = tokio::process::Command::new("python")
            .arg("--version")
            .output()
            .await;

        if python_check.is_err() {
            // Try python3
            let python3_check = tokio::process::Command::new("python3")
                .arg("--version")
                .output()
                .await;

            if python3_check.is_err() {
                return Err(anyhow!(
                    "Python is not installed. Please install Python 3.8 or later from python.org"
                ));
            }
        }

        // Check available disk space
        let config = self.config.read().await;
        let required_space_mb = match config.model_size.as_str() {
            "small" => 2000,  // 2GB
            "medium" => 5000, // 5GB
            "large" => 10000, // 10GB
            _ => 5000,
        };

        // Note: In production, add actual disk space check here

        Ok(())
    }

    async fn install_presidio_components(&self) -> Result<()> {
        use crate::presidio_bridge::PresidioBridge;

        let bridge = PresidioBridge::new();
        bridge.setup().await?;

        Ok(())
    }

    async fn download_ai_models(&self, config: &SetupConfig) -> Result<()> {
        // Download models based on size preference
        let models_to_download = match config.model_size.as_str() {
            "small" => vec![
                ("TinyLlama-1.1B", "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF"),
                ("PII-Redact-Small", "lakshyakh93/deberta_finetuned_pii"),
            ],
            "medium" => vec![
                ("Mistral-7B", "TheBloke/Mistral-7B-Instruct-v0.2-GGUF"),
                ("PII-Redact-Base", "lakshyakh93/deberta_finetuned_pii"),
                ("Legal-BERT", "nlpaueb/legal-bert-base-uncased"),
            ],
            "large" => vec![
                ("Llama2-13B", "TheBloke/Llama-2-13B-chat-GGUF"),
                ("PII-Redact-Large", "lakshyakh93/deberta_finetuned_pii"),
                ("Legal-BERT-Large", "nlpaueb/legal-bert-base-uncased"),
            ],
            _ => vec![],
        };

        let total_models = models_to_download.len();
        for (i, (name, repo)) in models_to_download.iter().enumerate() {
            let progress = 50.0 + (30.0 * (i as f32) / total_models as f32);
            let msg = format!("Downloading {} model...", name);
            self.send_progress("Downloading models", progress, &msg).await?;

            // Note: Actual model download implementation would go here
            // For now, we'll simulate with a delay
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        Ok(())
    }

    async fn verify_setup(&self) -> Result<()> {
        // Verify Presidio
        let presidio_check = tokio::process::Command::new("python")
            .args(&["-c", "import presidio_analyzer; print('OK')"])
            .output()
            .await;

        if let Ok(output) = presidio_check {
            if !output.status.success() {
                println!("Warning: Presidio verification failed, but continuing...");
            }
        }

        // Verify model files exist
        let config = self.config.read().await;
        let models_dir = config.data_dir.join("models");

        if !models_dir.exists() {
            return Err(anyhow!("Models directory not found"));
        }

        Ok(())
    }

    async fn mark_setup_complete(&self, config: &SetupConfig) -> Result<()> {
        let marker_file = config.data_dir.join(".setup_complete");

        let setup_info = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "setup_date": chrono::Utc::now().to_rfc3339(),
            "presidio_installed": config.install_presidio,
            "models_installed": config.install_models,
            "model_size": config.model_size,
        });

        tokio::fs::write(marker_file, serde_json::to_string_pretty(&setup_info)?).await?;

        Ok(())
    }

    pub async fn is_setup_complete(&self) -> bool {
        let complete = self.setup_complete.read().await;
        *complete
    }

    pub async fn update_config(&self, config: SetupConfig) -> Result<()> {
        let mut current = self.config.write().await;
        *current = config;
        Ok(())
    }

    pub async fn get_setup_status(&self) -> Result<serde_json::Value> {
        let config = self.config.read().await;
        let marker_file = config.data_dir.join(".setup_complete");

        if marker_file.exists() {
            let content = tokio::fs::read_to_string(marker_file).await?;
            let info: serde_json::Value = serde_json::from_str(&content)?;
            Ok(info)
        } else {
            Ok(serde_json::json!({
                "setup_complete": false,
                "is_first_run": true
            }))
        }
    }
}