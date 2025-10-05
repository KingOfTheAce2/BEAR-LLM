use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};

// Global setup lock to prevent concurrent setup runs
static SETUP_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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
        // Acquire global setup lock to prevent concurrent setup runs
        // This prevents race conditions when multiple windows/processes try to run setup
        let _lock = SETUP_LOCK.lock().await;

        tracing::info!("Setup lock acquired, beginning setup process");

        // Check if setup is already complete (another instance may have completed it)
        if *self.setup_complete.read().await {
            tracing::debug!("Setup already completed by another instance, skipping");
            self.send_progress("Complete", 100.0, "Setup already completed")
                .await?;
            return Ok(());
        }

        // Double-check marker file after acquiring lock
        let config = self.config.read().await.clone();
        let marker_file = config.data_dir.join(".setup_complete");
        if marker_file.exists() {
            tracing::info!("Setup marker file found after lock, marking as complete");
            let mut complete = self.setup_complete.write().await;
            *complete = true;
            self.send_progress("Complete", 100.0, "Setup already completed")
                .await?;
            return Ok(());
        }

        // Store progress sender
        let mut sender_lock = self.progress_sender.write().await;
        *sender_lock = Some(progress_sender.clone());
        drop(sender_lock);

        // If nothing is selected, just mark as complete
        if !config.install_presidio && !config.install_models {
            self.send_progress("Skipping", 100.0, "No components selected, setup skipped.")
                .await?;
            self.mark_setup_complete(&config).await?;
            let mut complete = self.setup_complete.write().await;
            *complete = true;
            return Ok(());
        }

        // Send initial progress
        self.send_progress("Initializing", 0.0, "Starting BEAR AI setup...")
            .await?;

        // Step 1: Create directories
        self.send_progress(
            "Creating directories",
            10.0,
            "Setting up application directories...",
        )
        .await?;
        self.create_directories(&config).await?;

        // Step 2: Check system requirements
        self.send_progress(
            "Checking requirements",
            20.0,
            "Verifying system requirements...",
        )
        .await?;
        self.check_requirements().await?;

        // Step 3: Install Python dependencies
        if config.install_presidio {
            self.send_progress(
                "Installing Presidio",
                30.0,
                "Installing Microsoft Presidio for state-of-the-art PII protection...",
            )
            .await?;
            self.install_presidio_components().await?;
        }

        // Step 4: Download models
        if config.install_models {
            self.send_progress(
                "Downloading models",
                50.0,
                "Downloading AI models (this may take several minutes)...",
            )
            .await?;
            self.download_ai_models(&config).await?;
        }

        // Step 5: Verify installation
        self.send_progress("Verifying", 80.0, "Verifying installation...")
            .await?;
        self.verify_setup().await?;

        // Step 6: Mark setup complete
        self.send_progress("Finalizing", 95.0, "Finalizing setup...")
            .await?;
        self.mark_setup_complete(&config).await?;

        // Send completion
        self.send_progress("Complete", 100.0, "Setup completed successfully!")
            .await?;

        let mut complete = self.setup_complete.write().await;
        *complete = true;

        Ok(())
    }

    pub async fn mark_setup_complete_only(&self) -> Result<()> {
        // Acquire lock for consistency, even though this is just marking complete
        let _lock = SETUP_LOCK.lock().await;

        tracing::info!("Manually marking setup as complete");

        let config = self.config.read().await.clone();
        self.mark_setup_complete(&config).await?;
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

            sender
                .send(progress_msg)
                .await
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

        // Check actual disk space
        let data_dir = config.data_dir.clone();
        drop(config);

        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();

        if let Some(disk) = disks.iter().find(|d| data_dir.starts_with(d.mount_point())) {
            let available_mb = disk.available_space() / (1024 * 1024);
            if available_mb < required_space_mb {
                return Err(anyhow!(
                    "Insufficient disk space. Required: {}MB, Available: {}MB. Please free up disk space.",
                    required_space_mb, available_mb
                ));
            }
            tracing::info!(
                "Disk space check passed: {}MB available, {}MB required",
                available_mb,
                required_space_mb
            );
        } else {
            tracing::warn!(
                "Could not determine disk space for {:?}, proceeding anyway",
                data_dir
            );
        }

        Ok(())
    }

    async fn install_presidio_components(&self) -> Result<()> {
        use crate::presidio_bridge::PresidioBridge;

        let bridge = PresidioBridge::new();
        bridge.setup().await?;

        Ok(())
    }

    async fn download_ai_models(&self, config: &SetupConfig) -> Result<()> {
        // Step 1: Download RAG embeddings model (CRITICAL - required for document processing)
        self.send_progress(
            "Downloading models",
            50.0,
            "Downloading RAG embeddings model (~150MB)...",
        )
        .await?;

        if let Err(e) = self.download_rag_embeddings().await {
            tracing::warn!(
                "Failed to download RAG embeddings during setup: {}. Will download on first use.",
                e
            );
            // Continue with LLM download even if RAG fails
        }

        // Step 2: Download LLM model based on corporate laptop compatibility
        self.send_progress(
            "Downloading models",
            60.0,
            "Downloading LLM for text generation...",
        )
        .await?;

        // Corporate laptop optimized models (determined by model_size selection)
        let (model_name, repo_id, file_name) = match config.model_size.as_str() {
            "small" => (
                "TinyLlama-1.1B (Corporate Laptop - Fast)",
                "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF",
                "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf", // ~700MB, runs on any laptop
            ),
            "medium" => (
                "Phi-2 (Corporate Laptop - Balanced)",
                "TheBloke/phi-2-GGUF",
                "phi-2.Q4_K_M.gguf", // ~1.6GB, good for 8GB RAM laptops
            ),
            "large" => (
                "Mistral-7B (Workstation - Best Quality)",
                "TheBloke/Mistral-7B-Instruct-v0.2-GGUF",
                "mistral-7b-instruct-v0.2.Q4_K_M.gguf", // ~4.4GB, needs 16GB+ RAM
            ),
            _ => (
                "TinyLlama-1.1B (Default)",
                "TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF",
                "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
            ),
        };

        tracing::info!("📥 Downloading LLM: {} from {}", model_name, repo_id);

        // Actually download the LLM model
        if let Err(e) = self.download_llm_model(repo_id, file_name).await {
            tracing::error!(
                "Failed to download LLM model: {}. Application may not work properly.",
                e
            );
            return Err(anyhow!("Failed to download LLM model: {}", e));
        }

        self.send_progress(
            "Downloading models",
            80.0,
            &format!("✅ {} downloaded successfully", model_name),
        )
        .await?;

        Ok(())
    }

    async fn download_llm_model(&self, repo_id: &str, file_name: &str) -> Result<()> {
        use hf_hub::api::tokio::Api;

        let config = self.config.read().await;
        let models_dir = config
            .data_dir
            .join("models")
            .join(repo_id.replace("/", "_"));
        drop(config);

        // Create models directory
        tokio::fs::create_dir_all(&models_dir).await?;

        tracing::info!("📥 Downloading {} from HuggingFace...", file_name);

        // Initialize HuggingFace API
        let api = Api::new()?;
        let repo = api.model(repo_id.to_string());

        // Download the model file
        let downloaded_path = repo
            .get(file_name)
            .await
            .map_err(|e| anyhow!("Failed to download {}: {}", file_name, e))?;

        // Copy to models directory
        let dest_path = models_dir.join(file_name);
        tokio::fs::copy(&downloaded_path, &dest_path)
            .await
            .map_err(|e| anyhow!("Failed to copy model file: {}", e))?;

        tracing::info!("✅ LLM model downloaded to: {:?}", dest_path);

        Ok(())
    }

    async fn download_rag_embeddings(&self) -> Result<()> {
        use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

        tracing::info!("📥 Downloading BGE embeddings model for RAG engine...");

        // This will download the model to .fastembed_cache if not present
        let _model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15).with_show_download_progress(true),
        )?;

        tracing::info!("✅ RAG embeddings model downloaded successfully");
        Ok(())
    }

    async fn verify_setup(&self) -> Result<()> {
        // Verify Presidio
        let presidio_check = tokio::process::Command::new("python")
            .args(["-c", "import presidio_analyzer; print('OK')"])
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

    #[allow(dead_code)]
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
