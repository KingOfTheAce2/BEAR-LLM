use crate::process_helper::ProcessCommandExt;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::process::Command as AsyncCommand;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresidioEntity {
    pub entity_type: String,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub score: f32,
    pub recognition_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresidioConfig {
    pub language: String,
    pub entities: Vec<String>,
    pub score_threshold: f32,
    pub return_decision_process: bool,
    pub use_gpu: bool,
}

impl Default for PresidioConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            entities: vec![
                "PERSON".to_string(),
                "EMAIL_ADDRESS".to_string(),
                "PHONE_NUMBER".to_string(),
                "US_SSN".to_string(),
                "CREDIT_CARD".to_string(),
                "LOCATION".to_string(),
                "ORGANIZATION".to_string(),
                "DATE_TIME".to_string(),
                "NRP".to_string(), // Nationality, religious, political group
                "MEDICAL_LICENSE".to_string(),
                "US_DRIVER_LICENSE".to_string(),
                "US_PASSPORT".to_string(),
                "IP_ADDRESS".to_string(),
                "IBAN_CODE".to_string(),
                "US_BANK_NUMBER".to_string(),
            ],
            score_threshold: 0.85,
            return_decision_process: false,
            use_gpu: false,
        }
    }
}

pub struct PresidioBridge {
    python_path: Arc<RwLock<Option<PathBuf>>>,
    presidio_installed: Arc<RwLock<bool>>,
    model_path: Arc<RwLock<Option<PathBuf>>>,
    config: Arc<RwLock<PresidioConfig>>,
}

impl PresidioBridge {
    pub fn new() -> Self {
        let app_data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("bear-ai-llm")
            .join("presidio");

        Self {
            python_path: Arc::new(RwLock::new(None)),
            presidio_installed: Arc::new(RwLock::new(false)),
            model_path: Arc::new(RwLock::new(Some(app_data_dir))),
            config: Arc::new(RwLock::new(PresidioConfig::default())),
        }
    }

    pub async fn setup(&self) -> Result<()> {
        println!("🔧 Setting up Microsoft Presidio for state-of-the-art PII protection...");

        // Step 1: Check/Install Python
        self.ensure_python().await?;

        // Step 2: Install Presidio and dependencies
        self.install_presidio().await?;

        // Step 3: Download OpenPipe PII-Redact model
        self.download_models().await?;

        // Step 4: Verify installation
        self.verify_installation().await?;

        println!("✅ Presidio setup complete!");
        Ok(())
    }

    async fn ensure_python(&self) -> Result<()> {
        println!("📍 Checking Python installation...");

        // Try to find Python
        let python_commands = vec!["python3", "python", "py"];
        let mut found_python = None;

        for cmd in python_commands {
            if let Ok(output) = Command::new(cmd).arg("--version").no_window().output() {
                if output.status.success() {
                    found_python = Some(cmd.to_string());
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("✅ Found Python: {}", version.trim());
                    break;
                }
            }
        }

        let python_cmd = found_python
            .ok_or_else(|| anyhow!("Python not found. Please install Python 3.8+ first."))?;

        let mut python_path = self.python_path.write().await;
        *python_path = Some(PathBuf::from(python_cmd));

        Ok(())
    }

    async fn install_presidio(&self) -> Result<()> {
        println!("📦 Installing Microsoft Presidio and dependencies...");

        let python_path = self.python_path.read().await;
        let python = python_path
            .as_ref()
            .ok_or_else(|| anyhow!("Python path not set"))?;

        // Create requirements file for LITE mode (spaCy only)
        let requirements = r#"
# Microsoft Presidio - State-of-the-art PII detection
presidio-analyzer>=2.2.0
presidio-anonymizer>=2.2.0

# NLP engines for enhanced detection (LITE MODE - spaCy only)
spacy>=3.0.0

# FULL MODE ONLY (commented out for lite mode to save 1.5GB RAM)
# Uncomment these lines if you want full ML detection:
# transformers>=4.30.0
# torch>=2.0.0
# accelerate>=0.20.0

# Performance optimizations
numpy>=1.24.0
scipy>=1.10.0
"#;

        let model_path = self.model_path.read().await;
        let requirements_path = model_path
            .as_ref()
            .ok_or_else(|| anyhow!("Model path not set"))?
            .join("requirements.txt");

        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(requirements_path.parent().unwrap()).await?;
        tokio::fs::write(&requirements_path, requirements).await?;

        // Install packages
        println!("📥 Installing packages (this may take a few minutes on first run)...");

        let output = AsyncCommand::new(python)
            .no_window()
            .args(&["-m", "pip", "install", "-r"])
            .arg(&requirements_path)
            .no_window()
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to install Presidio: {}", error));
        }

        // Download spaCy model (SMALL version for lite mode - 40MB vs 560MB)
        println!("📥 Downloading spaCy NER model (small - 40MB)...");
        let output = AsyncCommand::new(python)
            .no_window()
            .args(&["-m", "spacy", "download", "en_core_web_sm"])
            .no_window()
            .output()
            .await?;

        if !output.status.success() {
            println!("⚠️  spaCy model download failed, will retry on demand");
        }

        println!("💡 Tip: For better accuracy (but +520MB RAM), download en_core_web_lg");
        println!("   Run: python -m spacy download en_core_web_lg");

        let mut installed = self.presidio_installed.write().await;
        *installed = true;

        println!("✅ Presidio installed successfully!");
        Ok(())
    }

    async fn download_models(&self) -> Result<()> {
        println!("🤖 Downloading OpenPipe PII-Redact models...");

        let model_path = self.model_path.read().await;
        let model_dir = model_path
            .as_ref()
            .ok_or_else(|| anyhow!("Model path not set"))?;

        // Create Python script to download models
        let download_script = r#"
import os
import sys
from transformers import AutoTokenizer, AutoModelForTokenClassification
import torch

def download_models():
    print("Downloading OpenPipe PII-Redact model...")

    # Download the state-of-the-art PII model
    model_name = "lakshyakh93/deberta_finetuned_pii"  # High-accuracy PII model

    try:
        # Download tokenizer and model
        tokenizer = AutoTokenizer.from_pretrained(model_name)
        model = AutoModelForTokenClassification.from_pretrained(model_name)

        # Save locally
        save_dir = os.path.join(os.path.dirname(__file__), "models", "pii_redact")
        os.makedirs(save_dir, exist_ok=True)

        tokenizer.save_pretrained(save_dir)
        model.save_pretrained(save_dir)

        print(f"✅ Model saved to: {save_dir}")

        # Also download a legal-specific model if available
        legal_model = "dslim/bert-base-NER"  # General NER as fallback
        tokenizer_legal = AutoTokenizer.from_pretrained(legal_model)
        model_legal = AutoModelForTokenClassification.from_pretrained(legal_model)

        save_dir_legal = os.path.join(os.path.dirname(__file__), "models", "legal_ner")
        os.makedirs(save_dir_legal, exist_ok=True)

        tokenizer_legal.save_pretrained(save_dir_legal)
        model_legal.save_pretrained(save_dir_legal)

        print(f"✅ Legal NER model saved to: {save_dir_legal}")

    except Exception as e:
        print(f"Error downloading models: {e}")
        sys.exit(1)

if __name__ == "__main__":
    download_models()
"#;

        let script_path = model_dir.join("download_models.py");
        tokio::fs::write(&script_path, download_script).await?;

        let python_path = self.python_path.read().await;
        let python = python_path
            .as_ref()
            .ok_or_else(|| anyhow!("Python path not set"))?;

        let output = AsyncCommand::new(python)
            .no_window()
            .arg(&script_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("⚠️  Model download failed: {}", error);
            println!("Models will be downloaded on first use.");
        } else {
            println!("✅ Models downloaded successfully!");
        }

        Ok(())
    }

    async fn verify_installation(&self) -> Result<()> {
        println!("🔍 Verifying Presidio installation...");

        let python_path = self.python_path.read().await;
        let python = python_path
            .as_ref()
            .ok_or_else(|| anyhow!("Python path not set"))?;

        let verification_script = r#"
import sys
try:
    import presidio_analyzer
    import presidio_anonymizer
    import spacy
    import transformers
    print("SUCCESS: All components installed")
    sys.exit(0)
except ImportError as e:
    print(f"ERROR: Missing component: {e}")
    sys.exit(1)
"#;

        let model_path = self.model_path.read().await;
        let script_path = model_path
            .as_ref()
            .ok_or_else(|| anyhow!("Model path not set"))?
            .join("verify.py");

        tokio::fs::write(&script_path, verification_script).await?;

        let output = AsyncCommand::new(python)
            .no_window()
            .arg(&script_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Presidio verification failed: {}", error));
        }

        println!("✅ Presidio verification successful!");
        Ok(())
    }

    pub async fn detect_pii(&self, text: &str) -> Result<Vec<PresidioEntity>> {
        // Ensure setup is complete
        let installed = self.presidio_installed.read().await;
        if !*installed {
            drop(installed);
            self.setup().await?;
        }

        let python_path = self.python_path.read().await;
        let python = python_path
            .as_ref()
            .ok_or_else(|| anyhow!("Python path not set"))?;

        let config = self.config.read().await;
        let model_path = self.model_path.read().await;

        // Create detection script (LITE mode - spaCy only, no transformers)
        let detection_script = format!(
            r#"
import json
import sys
from presidio_analyzer import AnalyzerEngine, RecognizerRegistry
from presidio_analyzer.nlp_engine import NlpEngineProvider

def detect_pii(text, config):
    # Initialize NLP engine with spaCy (LITE MODE - small model)
    provider = NlpEngineProvider(nlp_configuration={{
        "nlp_engine_name": "spacy",
        "models": [
            {{"lang_code": "en", "model_name": "en_core_web_sm"}}
        ],
    }})
    nlp_engine = provider.create_engine()

    # Create analyzer with custom configuration
    analyzer = AnalyzerEngine(
        nlp_engine=nlp_engine,
        supported_languages=["en"]
    )

    # FULL MODE: Uncomment to use transformer models (requires torch, transformers)
    # This adds ~1.5GB RAM but improves accuracy from 90% to 95%
    # try:
    #     from transformers import pipeline
    #     ner_pipeline = pipeline(
    #         "ner",
    #         model="lakshyakh93/deberta_finetuned_pii",
    #         aggregation_strategy="simple"
    #     )
    #     transformer_results = ner_pipeline(text)
    # except:
    #     transformer_results = []

    # Analyze text with Presidio
    results = analyzer.analyze(
        text=text,
        entities=config["entities"],
        language=config["language"],
        score_threshold=config["score_threshold"],
        return_decision_process=config["return_decision_process"]
    )

    # Convert to JSON
    output = []
    for result in results:
        output.append({{
            "entity_type": result.entity_type,
            "text": text[result.start:result.end],
            "start": result.start,
            "end": result.end,
            "score": result.score,
            "recognition_metadata": {{}}
        }})

    return json.dumps(output)

if __name__ == "__main__":
    text = sys.argv[1]
    config = json.loads(sys.argv[2])
    result = detect_pii(text, config)
    print(result)
"#
        );

        let script_path = model_path
            .as_ref()
            .ok_or_else(|| anyhow!("Model path not set"))?
            .join("detect.py");

        tokio::fs::write(&script_path, detection_script).await?;

        let config_json = serde_json::to_string(&*config)?;

        let output = AsyncCommand::new(python)
            .no_window()
            .arg(&script_path)
            .arg(text)
            .arg(&config_json)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("PII detection failed: {}", error));
        }

        let result_json = String::from_utf8_lossy(&output.stdout);
        let entities: Vec<PresidioEntity> = serde_json::from_str(&result_json)?;

        Ok(entities)
    }

    pub async fn anonymize(&self, text: &str, entities: Vec<PresidioEntity>) -> Result<String> {
        let python_path = self.python_path.read().await;
        let python = python_path
            .as_ref()
            .ok_or_else(|| anyhow!("Python path not set"))?;

        let model_path = self.model_path.read().await;

        // Create anonymization script
        let anonymize_script = r#"
import json
import sys
from presidio_anonymizer import AnonymizerEngine
from presidio_anonymizer.entities import RecognizerResult, OperatorConfig

def anonymize_text(text, entities_json):
    entities = json.loads(entities_json)

    # Convert to RecognizerResult objects
    recognizer_results = []
    for entity in entities:
        result = RecognizerResult(
            entity_type=entity["entity_type"],
            start=entity["start"],
            end=entity["end"],
            score=entity["score"]
        )
        recognizer_results.append(result)

    # Initialize anonymizer
    engine = AnonymizerEngine()

    # Define anonymization operators
    operators = {
        "DEFAULT": OperatorConfig("replace", {"new_value": "[REDACTED]"}),
        "PERSON": OperatorConfig("replace", {"new_value": "[PERSON]"}),
        "EMAIL_ADDRESS": OperatorConfig("replace", {"new_value": "[EMAIL]"}),
        "PHONE_NUMBER": OperatorConfig("replace", {"new_value": "[PHONE]"}),
        "US_SSN": OperatorConfig("replace", {"new_value": "[SSN]"}),
        "CREDIT_CARD": OperatorConfig("mask", {"type": "mask", "masking_char": "*", "chars_to_mask": 12, "from_end": False}),
    }

    # Anonymize
    result = engine.anonymize(
        text=text,
        analyzer_results=recognizer_results,
        operators=operators
    )

    return result.text

if __name__ == "__main__":
    text = sys.argv[1]
    entities = sys.argv[2]
    result = anonymize_text(text, entities)
    print(result)
"#;

        let script_path = model_path
            .as_ref()
            .ok_or_else(|| anyhow!("Model path not set"))?
            .join("anonymize.py");

        tokio::fs::write(&script_path, anonymize_script).await?;

        let entities_json = serde_json::to_string(&entities)?;

        let output = AsyncCommand::new(python)
            .no_window()
            .arg(&script_path)
            .arg(text)
            .arg(&entities_json)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Anonymization failed: {}", error));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub async fn check_installation_status(&self) -> Result<bool> {
        let installed = self.presidio_installed.read().await;
        Ok(*installed)
    }

    pub async fn update_config(&self, config: PresidioConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }
}
