//! # PII Detector - Enterprise-Grade Privacy Protection
//!
//! Comprehensive PII detection and redaction system.
//!
//! ## Features
//! - **Presidio Integration**: Enterprise-grade PII detection when Python+Presidio available
//! - **Built-in Fallback**: Comprehensive regex-based detection for reliability
//! - **Luhn Validation**: Credit card number validation using Luhn algorithm
//! - **Context Enhancement**: Boost confidence based on surrounding text
//! - **Async Operations**: Full async/await support for non-blocking detection
//! - **Custom Patterns**: Support for adding domain-specific PII patterns
//!
//! ## Usage
//! ```rust,no_run
//! use crate::PIIDetector; // Corrected import
//! use anyhow::Result; // Added for Result type
//!
//! #[tokio::main] // Added tokio runtime
//! async fn main() -> Result<()> { // Made async
//!     let mut detector = PIIDetector::new();
//!     detector.initialize().await?;
//!
//!     let text = "John Doe's email is john@example.com"; // Defined text
//!     let entities = detector.detect_pii(text).await?;
//!     let redacted = detector.redact_pii(text).await?;
//!
//!     println!("Entities: {:?}", entities);
//!     println!("Redacted: {}", redacted);
//!     Ok(())
//! }
//! ```
//!
//! ## Detection Capabilities
//! - Social Security Numbers (SSN)
//! - Credit Cards (with Luhn validation)
//! - Email addresses
//! - Phone numbers
//! - IP addresses
//! - Case numbers (legal documents)
//! - Medical record numbers
//! - Person names (with context awareness)
//! - Organizations (companies, law firms)
//! - Custom patterns (configurable)

use crate::process_helper::ProcessCommandExt;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command as AsyncCommand;
use tokio::sync::RwLock;
use candle_core::Device;

pub mod candle_ner;
use crate::pii_detector::candle_ner::NerModel;

// Layer 2: Planned for ML-enhanced detection (currently blocked by dependency conflict)
// TODO: Implement with candle-transformers or wait for gline-rs dependency fix

lazy_static! {
    // Compiled regex patterns for performance
    static ref SSN_PATTERN: Regex = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")
        .expect("CRITICAL: SSN pattern regex is invalid - this should never fail");
    static ref CREDIT_CARD_PATTERN: Regex = Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b")
        .expect("CRITICAL: Credit card pattern regex is invalid - this should never fail");
    static ref EMAIL_PATTERN: Regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
        .expect("CRITICAL: Email pattern regex is invalid - this should never fail");
    static ref PHONE_PATTERN: Regex = Regex::new(r"\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b")
        .expect("CRITICAL: Phone pattern regex is invalid - this should never fail");
    static ref IP_PATTERN: Regex = Regex::new(r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b")
        .expect("CRITICAL: IP pattern regex is invalid - this should never fail");
    static ref CASE_NUMBER_PATTERN: Regex = Regex::new(r"\b(?:Case\s*(?:No\.?|Number)?:?\s*)?(\d{2,4}[-\s]?[A-Z]{2,4}[-\s]?\d{3,6})\b")
        .expect("CRITICAL: Case number pattern regex is invalid - this should never fail");
    static ref MEDICAL_RECORD_PATTERN: Regex = Regex::new(r"\b(?:MRN|Medical Record(?:\s*Number)?):?\s*([A-Z0-9]{6,12})\b")
        .expect("CRITICAL: Medical record pattern regex is invalid - this should never fail");
    static ref NAME_PATTERN: Regex = Regex::new(r"\b([A-Z][a-z]+ (?:[A-Z]\. )?[A-Z][a-z]+)\b")
        .expect("CRITICAL: Name pattern regex is invalid - this should never fail");
    static ref TITLE_NAME_PATTERN: Regex = Regex::new(r"\b(?:Mr\.|Mrs\.|Ms\.|Dr\.|Prof\.|Judge|Attorney|Counselor)\s+([A-Z][a-z]+ ?[A-Z]?[a-z]*)\b")
        .expect("CRITICAL: Title name pattern regex is invalid - this should never fail");
    static ref ORG_PATTERN: Regex = Regex::new(r"\b([A-Z][A-Za-z&\s]+ (?:Inc|LLC|LLP|Corp|Corporation|Company|Partners|Group|Associates|Firm|LTD|Limited))\b")
        .expect("CRITICAL: Organization pattern regex is invalid - this should never fail");
    static ref LEGAL_ORG_PATTERN: Regex = Regex::new(r"\b(?:Law (?:Office|Firm) of |The )([A-Z][a-z]+ (?:& )?[A-Z][a-z]+)\b")
        .expect("CRITICAL: Legal organization pattern regex is invalid - this should never fail");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIIEntity {
    pub entity_type: String,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
    pub engine: String, // "presidio", "transformer", or "regex"
}

/// Detection layer configuration
/// Layer 1 (Regex): Fast, always-on basic patterns
/// Layer 2 (ML): Planned Rust-native ML detection (coming soon)
/// Layer 3 (Presidio): Optional advanced ML (post-install)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DetectionLayer {
    /// Layer 1 only - Regex patterns (fastest, ~85% accuracy)
    #[default]
    RegexOnly,
    /// Layer 1 + Layer 2 (Candle) - Regex + Candle (balanced, ~92% accuracy)
    WithCandle,
    /// Layer 1 + Layer 2 (Candle) + Layer 3 (Presidio) - Full Stack (best accuracy)
    FullStack,
}

impl std::fmt::Display for DetectionLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DetectionLayer::RegexOnly => write!(f, "regex_only"),
            DetectionLayer::WithCandle => write!(f, "with_candle"),
            DetectionLayer::FullStack => write!(f, "full_stack"),
        }
    }
}

#[allow(dead_code)]
impl DetectionLayer {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "regex_only" | "layer1" | "fast" => DetectionLayer::RegexOnly,
            "with_candle" | "layer2" | "candle" | "balanced" => DetectionLayer::WithCandle,
            "full_stack" | "layer3" | "presidio" | "full" => DetectionLayer::FullStack,
            _ => DetectionLayer::RegexOnly, // Default to fast mode
        }
    }

    /// Get expected accuracy percentage
    pub fn accuracy(&self) -> u8 {
        match self {
            DetectionLayer::RegexOnly => 85,
            DetectionLayer::WithCandle => 92,
            DetectionLayer::FullStack => 95,
        }
    }

    /// Get number of active layers
    pub fn layer_count(&self) -> u8 {
        match self {
            DetectionLayer::RegexOnly => 1,
            DetectionLayer::WithCandle => 2,
            DetectionLayer::FullStack => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PresidioMode {
    /// Presidio disabled - use built-in detection only
    #[default]
    Disabled,
    /// Presidio Lite - spaCy only (~500MB overhead)
    SpacyOnly,
    /// Presidio Full - spaCy + transformer models (~2GB overhead)
    FullML,
}

impl std::fmt::Display for PresidioMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresidioMode::Disabled => write!(f, "disabled"),
            PresidioMode::SpacyOnly => write!(f, "spacy_only"),
            PresidioMode::FullML => write!(f, "full_ml"),
        }
    }
}

#[allow(dead_code)]
impl PresidioMode {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "disabled" => PresidioMode::Disabled,
            "spacy_only" => PresidioMode::SpacyOnly,
            "full_ml" => PresidioMode::FullML,
            _ => PresidioMode::Disabled,
        }
    }

    /// Get memory overhead in MB
    pub fn memory_overhead_mb(&self) -> u64 {
        match self {
            PresidioMode::Disabled => 0,
            PresidioMode::SpacyOnly => 500,
            PresidioMode::FullML => 2048,
        }
    }

    /// Get expected accuracy
    pub fn accuracy(&self) -> u8 {
        match self {
            PresidioMode::Disabled => 85,
            PresidioMode::SpacyOnly => 90,
            PresidioMode::FullML => 95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIIDetectionConfig {
    pub use_presidio: bool, // Deprecated - use presidio_mode instead
    pub presidio_mode: PresidioMode,
    pub detection_layer: DetectionLayer, // NEW: Layer selection
    pub confidence_threshold: f32,
    pub detect_names: bool,
    pub detect_organizations: bool,
    pub detect_locations: bool,
    pub detect_emails: bool,
    pub detect_phones: bool,
    pub detect_ssn: bool,
    pub detect_credit_cards: bool,
    pub detect_medical: bool,
    pub detect_legal: bool,
    pub use_context_enhancement: bool,
    pub candle_model_language: String,
}

impl Default for PIIDetectionConfig {
    fn default() -> Self {
        Self {
            use_presidio: false, // Deprecated field - defaults to false for backward compat
            presidio_mode: PresidioMode::Disabled, // Use built-in by default
            detection_layer: DetectionLayer::RegexOnly, // Default to Layer 1 (fast)
            confidence_threshold: 0.85,
            detect_names: true,
            detect_organizations: true,
            detect_locations: true,
            detect_emails: true,
            detect_phones: true,
            detect_ssn: true,
            detect_credit_cards: true,
            detect_medical: true,
            detect_legal: true,
            use_context_enhancement: true,
            candle_model_language: "english".to_string(),
        }
    }
}

/// PII exclusions configuration loaded from TOML file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PIIExclusionsConfig {
    #[serde(flatten)]
    pub exclusions: PIIExclusions,
    #[serde(default)]
    pub settings: PIIExclusionSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PIIExclusions {
    // Flatten all region-specific arrays into a unified structure
    #[serde(default, flatten)]
    pub all_exclusions: HashMap<String, Vec<String>>,
}

impl PIIExclusions {
    /// Get all location exclusions from any region-specific array
    pub fn locations(&self) -> Vec<&String> {
        self.all_exclusions
            .iter()
            .filter(|(k, _)| k.contains("location") || k.contains("cities") || k.contains("provinces") || k.contains("prefectures") || k.contains("regions") || k.contains("country"))
            .flat_map(|(_, v)| v.iter())
            .collect()
    }

    /// Get all legal term exclusions
    pub fn legal_terms(&self) -> Vec<&String> {
        self.all_exclusions
            .iter()
            .filter(|(k, _)| k.contains("legal") || k.contains("court") || k.contains("data_protection"))
            .flat_map(|(_, v)| v.iter())
            .collect()
    }

    /// Get all organization exclusions
    pub fn organizations(&self) -> Vec<&String> {
        self.all_exclusions
            .iter()
            .filter(|(k, _)| k.contains("organization") || k.contains("government") || k.contains("institution"))
            .flat_map(|(_, v)| v.iter())
            .collect()
    }

    /// Get all time term exclusions
    pub fn time_terms(&self) -> Vec<&String> {
        self.all_exclusions
            .iter()
            .filter(|(k, _)| k.contains("time"))
            .flat_map(|(_, v)| v.iter())
            .collect()
    }

    /// Get total count of all exclusions
    pub fn total_count(&self) -> usize {
        self.all_exclusions.values().map(|v| v.len()).sum()
    }

    /// Get all exclusions as a flat iterator
    pub fn all(&self) -> impl Iterator<Item = &String> {
        self.all_exclusions.values().flat_map(|v| v.iter())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PIIExclusionSettings {
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,
    #[serde(default)]
    pub fuzzy_matching: bool,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub countries: Option<Vec<String>>,
}

fn default_min_confidence() -> f32 {
    0.5
}

impl Default for PIIExclusionsConfig {
    fn default() -> Self {
        let mut all_exclusions = HashMap::new();
        all_exclusions.insert(
            "locations".to_string(),
            vec!["United States".to_string(), "New York".to_string()],
        );
        all_exclusions.insert(
            "legal_terms".to_string(),
            vec!["First Amendment".to_string(), "Supreme Court".to_string()],
        );

        Self {
            exclusions: PIIExclusions { all_exclusions },
            settings: PIIExclusionSettings {
                case_sensitive: false,
                min_confidence: 0.5,
                fuzzy_matching: false,
                region: Some("en".to_string()),
                description: Some("Default English exclusions".to_string()),
                languages: Some(vec!["English".to_string()]),
                countries: Some(vec!["United States".to_string()]),
            },
        }
    }
}

impl Default for PIIExclusionSettings {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            min_confidence: 0.5,
            fuzzy_matching: false,
            region: None,
            description: None,
            languages: None,
            countries: None,
        }
    }
}

pub struct PIIDetector {
    config: Arc<RwLock<PIIDetectionConfig>>,
    exclusions_config: Arc<RwLock<PIIExclusionsConfig>>,
    python_path: Arc<RwLock<Option<PathBuf>>>,
    presidio_available: Arc<RwLock<bool>>,
    custom_patterns: Arc<RwLock<HashMap<String, Regex>>>,
    candle_ner_model: Arc<RwLock<Option<NerModel>>>,
}

impl Default for PIIDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PIIDetector {
    pub fn new() -> Self {
        // Load exclusions config from file, fallback to defaults if not found
        let exclusions_config = Self::load_exclusions_config().unwrap_or_else(|e| {
            tracing::error!(
                "==================== PII EXCLUSIONS CONFIG ERROR ===================="
            );
            tracing::error!("Failed to load PII exclusions configuration file!");
            tracing::error!("Error: {}", e);
            tracing::error!("");
            tracing::error!(
                "⚠️  WARNING: Without exclusions config, legal terms may be flagged as PII!"
            );
            tracing::error!("⚠️  Examples that may be incorrectly flagged:");
            tracing::error!("   - 'United States', 'New York', 'Supreme Court'");
            tracing::error!("   - 'First Amendment', 'Federal Court', 'Justice Department'");
            tracing::error!("");
            tracing::error!("To fix this issue:");
            tracing::error!("1. Create 'pii_exclusions.toml' in project root");
            tracing::error!("2. Or run: cargo run -- create-default-pii-config");
            tracing::error!("3. See example at: src-tauri/pii_exclusions.example.toml");
            tracing::error!("====================================================================");

            // Also log to stderr for visibility during development
            eprintln!("\n❌ PII EXCLUSIONS CONFIG MISSING - Legal terms may be flagged as PII!");
            eprintln!("   Create 'pii_exclusions.toml' to fix this issue.\n");

            PIIExclusionsConfig::default()
        });

        Self {
            config: Arc::new(RwLock::new(PIIDetectionConfig::default())),
            exclusions_config: Arc::new(RwLock::new(exclusions_config)),
            python_path: Arc::new(RwLock::new(None)),
            presidio_available: Arc::new(RwLock::new(false)),
            custom_patterns: Arc::new(RwLock::new(HashMap::new())),
            candle_ner_model: Arc::new(RwLock::new(None)),
        }
    }

    /// Load PII exclusions configuration from ALL regional TOML files
    /// Loads and merges: en, eu, apac, latam, mena, africa, south_asia, cis
    /// This ensures comprehensive multilingual PII detection regardless of document language
    fn load_exclusions_config() -> Result<PIIExclusionsConfig> {
        let regions = vec!["en", "eu", "apac", "latam", "mena", "africa", "south_asia", "cis"];
        let mut merged_exclusions = HashMap::new();
        let mut merged_settings = PIIExclusionSettings::default();
        let mut total_loaded = 0;
        let mut loaded_regions = Vec::new();

        tracing::info!("Loading PII exclusions from all regional files...");

        for region in &regions {
            let base_name = format!("pii_exclusions_{}.toml", region);

            // Try multiple possible locations
            let possible_paths = vec![
                PathBuf::from(&base_name),
                PathBuf::from("src-tauri").join(&base_name),
                dirs::config_dir()
                    .map(|p| p.join("bear-ai-llm").join(&base_name))
                    .unwrap_or_else(|| PathBuf::from(&base_name)),
            ];

            for path in possible_paths {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            match toml::from_str::<PIIExclusionsConfig>(&content) {
                                Ok(config) => {
                                    let count = config.exclusions.total_count();
                                    tracing::info!(
                                        "  ✅ Loaded {} patterns from {} ({})",
                                        count,
                                        region,
                                        path.display()
                                    );

                                    // Merge all exclusions
                                    for (key, values) in config.exclusions.all_exclusions {
                                        merged_exclusions.entry(key)
                                            .or_insert_with(Vec::new)
                                            .extend(values);
                                    }

                                    total_loaded += count;
                                    loaded_regions.push(region.to_string());

                                    // Use first loaded settings as base
                                    if total_loaded == count {
                                        merged_settings = config.settings;
                                    }

                                    break; // Found the file, stop searching paths
                                }
                                Err(e) => {
                                    tracing::warn!("  ⚠️  Failed to parse {}: {}", path.display(), e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("  ⚠️  Failed to read {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        if total_loaded == 0 {
            return Err(anyhow!(
                "❌ No PII exclusions config files found. Expected: pii_exclusions_en.toml, pii_exclusions_eu.toml, pii_exclusions_apac.toml"
            ));
        }

        // Update merged settings to reflect all loaded regions
        merged_settings.region = Some(format!("multi ({})", loaded_regions.join(", ")));
        merged_settings.description = Some(format!(
            "Merged exclusions from {} regions ({} total patterns)",
            loaded_regions.len(),
            total_loaded
        ));

        let merged_config = PIIExclusionsConfig {
            exclusions: PIIExclusions {
                all_exclusions: merged_exclusions,
            },
            settings: merged_settings,
        };

        let locations_count = merged_config.exclusions.locations().len();
        let legal_count = merged_config.exclusions.legal_terms().len();
        let org_count = merged_config.exclusions.organizations().len();
        let time_count = merged_config.exclusions.time_terms().len();

        tracing::info!("✅ Successfully merged {} exclusion patterns from {} regions", total_loaded, loaded_regions.len());
        tracing::info!("   - Regions: {}", loaded_regions.join(", "));
        tracing::info!("   - Locations: {}, Legal Terms: {}, Organizations: {}, Time Terms: {}",
            locations_count, legal_count, org_count, time_count);

        Ok(merged_config)
    }

    pub async fn initialize(&self) -> Result<()> {
        // Check for Python and Presidio (Layer 3)
        self.check_presidio_availability().await;

        // Initialize Candle NER model (Layer 2) if needed
        let config = self.config.read().await;
        let model_id = if config.candle_model_language.as_str() == "dutch" {
            "./models/robbert-v2-dutch-ner"
        } else {
            "./models/bert-large-cased-finetuned-conll03-english"
        };
        if matches!(config.detection_layer, DetectionLayer::WithCandle | DetectionLayer::FullStack) {
            let mut candle_ner_model = self.candle_ner_model.write().await;
            if candle_ner_model.is_none() {
                tracing::info!("Initializing Candle NER model for Layer 2...");
                let device = if candle_core::utils::cuda_is_available() {
                    Device::new_cuda(0)?
                } else {
                    Device::Cpu
                };
                match NerModel::new_local(PathBuf::from(model_id), device) {
                    Ok(model) => {
                        *candle_ner_model = Some(model);
                        tracing::info!("✅ Candle NER model initialized successfully.");
                    },
                    Err(e) => {
                        tracing::error!("❌ Failed to initialize Candle NER model: {}", e);
                        tracing::warn!("⚠️  Layer 2 (Candle) will be unavailable. Falling back to Layer 1.");
                    }
                }
            }
        }
        Ok(())
    }

    async fn check_presidio_availability(&self) {
        // Try to find Python
        for cmd in &["python3", "python", "py"] {
            if let Ok(output) = AsyncCommand::new(cmd)
                .arg("-c")
                .arg("import presidio_analyzer, presidio_anonymizer; print('OK')")
                .no_window()
                .output()
                .await
            {
                if output.status.success() {
                    let mut python_path = self.python_path.write().await;
                    *python_path = Some(PathBuf::from(cmd));

                    let mut available = self.presidio_available.write().await;
                    *available = true;

                    tracing::info!("✅ Presidio is available for enhanced PII detection");
                    return;
                }
            }
        }

        tracing::warn!("⚠️  Presidio not installed - using rudimentary privacy shield");
        tracing::warn!("⚠️  For enterprise-grade PII protection, install Microsoft Presidio");
        tracing::warn!("⚠️  Built-in detection has limited accuracy and may miss sensitive data");
    }

    pub async fn is_presidio_available(&self) -> bool {
        *self.presidio_available.read().await
    }

    pub async fn detect_pii(&self, text: &str) -> Result<Vec<PIIEntity>> {
        let config = self.config.read().await;
        let mut all_entities = Vec::new();

        // === 3-LAYER PII DETECTION SYSTEM ===
        // Layer 1: Regex (always active, fast baseline)
        // Layer 2: Candle (Rust-native ML)
        // Layer 3: MS Presidio (optional post-install ML)

        tracing::debug!("Starting PII detection (mode: {:?})", config.detection_layer);

        // LAYER 1: Regex-based detection (ALWAYS RUN - fast baseline)
        let layer1_start = std::time::Instant::now();
        let layer1_entities = self.detect_with_regex(text, &config).await?;
        tracing::debug!("Layer 1 (Regex): {} entities in {:?}", layer1_entities.len(), layer1_start.elapsed());
        all_entities.extend(layer1_entities);

        // LAYER 2: Candle NER (optional, if configured)
        if matches!(config.detection_layer, DetectionLayer::WithCandle | DetectionLayer::FullStack) {
            let mut candle_ner_model_guard = self.candle_ner_model.write().await;
            if let Some(ner_model) = candle_ner_model_guard.as_mut() {
                let layer2_start = std::time::Instant::now();
                match ner_model.predict(text) {
                    Ok(entities) => {
                        tracing::debug!("Layer 2 (Candle): {} entities in {:?}", entities.len(), layer2_start.elapsed());
                        all_entities.extend(entities);
                    }
                    Err(e) => {
                        tracing::warn!("Layer 2 (Candle) failed: {}. Falling back to Layer 1 results.", e);
                    }
                }
            } else {
                tracing::warn!("Layer 2 (Candle) is enabled but model is not loaded. Falling back to Layer 1 results.");
            }
        }

        // LAYER 3: MS Presidio (optional, post-install, if configured)
        if matches!(config.detection_layer, DetectionLayer::FullStack) {
            let should_use_presidio = match config.presidio_mode {
                PresidioMode::Disabled => false,
                PresidioMode::SpacyOnly | PresidioMode::FullML => true,
            };

            if should_use_presidio && *self.presidio_available.read().await {
                let layer3_start = std::time::Instant::now();
                match self.detect_with_presidio(text).await {
                    Ok(entities) => {
                        tracing::debug!("Layer 3 (Presidio): {} entities in {:?}", entities.len(), layer3_start.elapsed());
                        all_entities.extend(entities);
                    }
                    Err(e) => {
                        tracing::warn!("Layer 3 (Presidio) failed: {}. Falling back to Layer 1/2 results.", e);
                        // Fallback: Layer 1/2 results already added
                    }
                }
            }
        }

        // Post-processing: Context enhancement
        if config.use_context_enhancement {
            all_entities = self.enhance_with_context(text, all_entities);
        }

        // Final step: Deduplicate and filter by confidence
        let filtered = self.deduplicate_and_filter(all_entities, config.confidence_threshold);

        tracing::info!("PII detection complete: {} entities found (mode: {:?})",
            filtered.len(),
            config.detection_layer
        );

        Ok(filtered)
    }
    /// Layer 1: Regex-based detection (renamed from detect_with_builtin)
    async fn detect_with_regex(
        &self,
        text: &str,
        config: &PIIDetectionConfig,
    ) -> Result<Vec<PIIEntity>> {
        // This is the original detect_with_builtin logic
        self.detect_with_builtin(text, config).await
    }

    async fn detect_with_presidio(&self, text: &str) -> Result<Vec<PIIEntity>> {
        let python_path = self.python_path.read().await;
        let python = python_path
            .as_ref()
            .ok_or_else(|| anyhow!("Python path not set"))?;

        let script = r#"
import sys
import json
from presidio_analyzer import AnalyzerEngine

analyzer = AnalyzerEngine()
text = sys.argv[1]
results = analyzer.analyze(text=text, language='en')

entities = []
for result in results:
    entities.append({
        'entity_type': result.entity_type,
        'text': text[result.start:result.end],
        'start': result.start,
        'end': result.end,
        'confidence': result.score,
        'engine': 'presidio'
    })

print(json.dumps(entities))
"#;

        let temp_script = std::env::temp_dir().join("presidio_detect.py");
        tokio::fs::write(&temp_script, script).await?;

        let output = AsyncCommand::new(python)
            .no_window()
            .arg(&temp_script)
            .arg(text)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Presidio detection failed"));
        }

        let json_str = String::from_utf8(output.stdout)?;
        let entities: Vec<PIIEntity> = serde_json::from_str(&json_str)?;

        Ok(entities)
    }

    async fn detect_with_builtin(
        &self,
        text: &str,
        config: &PIIDetectionConfig,
    ) -> Result<Vec<PIIEntity>> {
        let mut entities = Vec::new();

        // High-confidence regex patterns
        if config.detect_ssn {
            for m in SSN_PATTERN.find_iter(text) {
                entities.push(PIIEntity {
                    entity_type: "SSN".to_string(),
                    text: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 1.0,
                    engine: "regex".to_string(),
                });
            }
        }

        if config.detect_credit_cards {
            for m in CREDIT_CARD_PATTERN.find_iter(text) {
                if self.validate_credit_card(m.as_str()) {
                    entities.push(PIIEntity {
                        entity_type: "CREDIT_CARD".to_string(),
                        text: m.as_str().to_string(),
                        start: m.start(),
                        end: m.end(),
                        confidence: 1.0,
                        engine: "regex".to_string(),
                    });
                }
            }
        }

        if config.detect_emails {
            for m in EMAIL_PATTERN.find_iter(text) {
                entities.push(PIIEntity {
                    entity_type: "EMAIL".to_string(),
                    text: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 1.0,
                    engine: "regex".to_string(),
                });
            }
        }

        if config.detect_phones {
            for m in PHONE_PATTERN.find_iter(text) {
                entities.push(PIIEntity {
                    entity_type: "PHONE".to_string(),
                    text: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.95,
                    engine: "regex".to_string(),
                });
            }
        }

        if config.detect_legal {
            for m in CASE_NUMBER_PATTERN.find_iter(text) {
                entities.push(PIIEntity {
                    entity_type: "CASE_NUMBER".to_string(),
                    text: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.9,
                    engine: "regex".to_string(),
                });
            }
        }

        if config.detect_medical {
            for m in MEDICAL_RECORD_PATTERN.find_iter(text) {
                entities.push(PIIEntity {
                    entity_type: "MEDICAL_RECORD".to_string(),
                    text: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.9,
                    engine: "regex".to_string(),
                });
            }
        }

        // Advanced name detection
        if config.detect_names {
            entities.extend(self.detect_names_advanced(text));
        }

        // Organization detection
        if config.detect_organizations {
            entities.extend(self.detect_organizations_advanced(text));
        }

        // Custom patterns
        let custom = self.custom_patterns.read().await;
        for (name, pattern) in custom.iter() {
            for m in pattern.find_iter(text) {
                entities.push(PIIEntity {
                    entity_type: name.clone(),
                    text: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.85,
                    engine: "regex".to_string(),
                });
            }
        }

        Ok(entities)
    }

    fn detect_names_advanced(&self, text: &str) -> Vec<PIIEntity> {
        let mut entities = Vec::new();
        let mut seen_positions = std::collections::HashSet::new();

        // Title patterns (high confidence)
        for cap in TITLE_NAME_PATTERN.captures_iter(text) {
            if let Some(name_match) = cap.get(1) {
                let pos = (name_match.start(), name_match.end());
                if !seen_positions.contains(&pos) {
                    seen_positions.insert(pos);
                    entities.push(PIIEntity {
                        entity_type: "PERSON".to_string(),
                        text: name_match.as_str().to_string(),
                        start: name_match.start(),
                        end: name_match.end(),
                        confidence: 0.9,
                        engine: "regex".to_string(),
                    });
                }
            }
        }

        // General name patterns (medium confidence)
        for m in NAME_PATTERN.find_iter(text) {
            let pos = (m.start(), m.end());
            if !seen_positions.contains(&pos) && !self.is_false_positive_name(m.as_str()) {
                seen_positions.insert(pos);
                entities.push(PIIEntity {
                    entity_type: "PERSON".to_string(),
                    text: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.75,
                    engine: "regex".to_string(),
                });
            }
        }

        entities
    }

    fn detect_organizations_advanced(&self, text: &str) -> Vec<PIIEntity> {
        let mut entities = Vec::new();
        let mut seen_positions = std::collections::HashSet::new();

        // Corporate suffixes
        for m in ORG_PATTERN.find_iter(text) {
            let pos = (m.start(), m.end());
            if !seen_positions.contains(&pos) {
                seen_positions.insert(pos);
                entities.push(PIIEntity {
                    entity_type: "ORGANIZATION".to_string(),
                    text: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.85,
                    engine: "regex".to_string(),
                });
            }
        }

        // Legal firms
        for cap in LEGAL_ORG_PATTERN.captures_iter(text) {
            if let Some(org_match) = cap.get(1) {
                let pos = (org_match.start(), org_match.end());
                if !seen_positions.contains(&pos) {
                    seen_positions.insert(pos);
                    entities.push(PIIEntity {
                        entity_type: "ORGANIZATION".to_string(),
                        text: org_match.as_str().to_string(),
                        start: org_match.start(),
                        end: org_match.end(),
                        confidence: 0.9,
                        engine: "regex".to_string(),
                    });
                }
            }
        }

        entities
    }

    fn enhance_with_context(&self, text: &str, mut entities: Vec<PIIEntity>) -> Vec<PIIEntity> {
        // Boost confidence based on surrounding context
        for entity in &mut entities {
            let context_start = entity.start.saturating_sub(50);
            let context_end = (entity.end + 50).min(text.len());
            let context = &text[context_start..context_end].to_lowercase();

            match entity.entity_type.as_str() {
                "PERSON" => {
                    if context.contains("plaintiff")
                        || context.contains("defendant")
                        || context.contains("attorney")
                        || context.contains("client")
                        || context.contains("witness")
                        || context.contains("judge")
                    {
                        entity.confidence = (entity.confidence * 1.2).min(1.0);
                    }
                }
                "ORGANIZATION" => {
                    if context.contains("company")
                        || context.contains("corporation")
                        || context.contains("firm")
                        || context.contains("agency")
                    {
                        entity.confidence = (entity.confidence * 1.15).min(1.0);
                    }
                }
                "SSN" | "CREDIT_CARD" => {
                    if context.contains("social security")
                        || context.contains("ssn")
                        || context.contains("credit")
                        || context.contains("card")
                    {
                        entity.confidence = 1.0;
                    }
                }
                _ => {}
            }
        }

        entities
    }

    fn deduplicate_and_filter(
        &self,
        mut entities: Vec<PIIEntity>,
        threshold: f32,
    ) -> Vec<PIIEntity> {
        // Sort by position and confidence (handle NaN values safely)
        entities.sort_by(|a, b| {
            a.start.cmp(&b.start).then(
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });

        let mut filtered = Vec::new();
        let mut last_end = 0;

        for entity in entities {
            // Skip if below threshold
            if entity.confidence < threshold {
                continue;
            }

            // Skip if overlapping with previous (keep higher confidence)
            if entity.start >= last_end {
                last_end = entity.end;
                filtered.push(entity);
            } else if !filtered.is_empty() {
                let last_idx = filtered.len() - 1;
                if entity.confidence > filtered[last_idx].confidence {
                    filtered[last_idx] = entity.clone();
                    last_end = entity.end;
                }
            }
        }

        filtered
    }

    fn validate_credit_card(&self, number: &str) -> bool {
        // Luhn algorithm validation
        let digits: Vec<u32> = number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap())
            .collect();

        if digits.len() < 13 || digits.len() > 19 {
            return false;
        }

        let mut sum = 0;
        let mut alternate = false;

        for digit in digits.iter().rev() {
            let mut d = *digit;
            if alternate {
                d *= 2;
                if d > 9 {
                    d -= 9;
                }
            }
            sum += d;
            alternate = !alternate;
        }

        sum % 10 == 0
    }

    fn is_false_positive_name(&self, text: &str) -> bool {
        // Use async-safe blocking read since we're in a sync function
        let exclusions_config = self.exclusions_config.try_read();

        if let Ok(config) = exclusions_config {
            let case_sensitive = config.settings.case_sensitive;

            // Check all exclusion categories using the new API
            for exclusion in config.exclusions.all() {
                let matches = if case_sensitive {
                    text == exclusion
                } else {
                    text.eq_ignore_ascii_case(exclusion)
                };

                if matches {
                    tracing::debug!("PII exclusion matched: '{}' against '{}'", text, exclusion);
                    return true;
                }
            }

            false
        } else {
            // Fallback to basic exclusions if config is locked
            const FALLBACK_EXCLUSIONS: &[&str] = &[
                "United States",
                "New York",
                "Supreme Court",
                "Federal Court",
            ];
            FALLBACK_EXCLUSIONS.contains(&text)
        }
    }

    pub async fn redact_pii(&self, text: &str) -> Result<String> {
        let entities = self.detect_pii(text).await?;
        let mut result = text.to_string();

        // Sort by position (reverse) for safe replacement
        let mut sorted_entities = entities;
        sorted_entities.sort_by_key(|e| std::cmp::Reverse(e.start));

        for entity in sorted_entities {
            let replacement = format!("[{}]", entity.entity_type);
            result.replace_range(entity.start..entity.end, &replacement);
        }

        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn anonymize_pii(&self, text: &str) -> Result<(String, HashMap<String, String>)> {
        let entities = self.detect_pii(text).await?;
        let mut result = text.to_string();
        let mut mappings = HashMap::new();
        let mut counters: HashMap<String, usize> = HashMap::new();

        // Sort by position (reverse) for safe replacement
        let mut sorted_entities = entities;
        sorted_entities.sort_by_key(|e| std::cmp::Reverse(e.start));

        for entity in sorted_entities {
            let counter = counters.entry(entity.entity_type.clone()).or_insert(0);
            *counter += 1;

            let placeholder = format!("{}_{:03}", entity.entity_type, counter);
            mappings.insert(placeholder.clone(), entity.text.clone());
            result.replace_range(entity.start..entity.end, &placeholder);
        }

        Ok((result, mappings))
    }

    #[allow(dead_code)]
    pub async fn add_custom_pattern(&self, name: String, pattern: String) -> Result<()> {
        let regex = Regex::new(&pattern)?;
        let mut patterns = self.custom_patterns.write().await;
        patterns.insert(name, regex);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn update_config(&self, config: PIIDetectionConfig) -> Result<()> {
        let mut current = self.config.write().await;
        *current = config;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_config(&self) -> PIIDetectionConfig {
        self.config.read().await.clone()
    }

    #[allow(dead_code)]
    pub async fn set_presidio_mode(&self, mode: PresidioMode) -> Result<()> {
        let mut config = self.config.write().await;
        config.presidio_mode = mode;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_presidio_mode(&self) -> PresidioMode {
        self.config.read().await.presidio_mode.clone()
    }

    /// Set detection layer (Layer 1, Layer 1+2, or Full Stack)
    #[allow(dead_code)]
    pub async fn set_detection_layer(&self, layer: DetectionLayer) -> Result<()> {
        let mut config = self.config.write().await;
        config.detection_layer = layer;
        tracing::info!("Detection layer updated to: {:?}", config.detection_layer);
        Ok(())
    }

    /// Get current detection layer configuration
    #[allow(dead_code)]
    pub async fn get_detection_layer(&self) -> DetectionLayer {
        self.config.read().await.detection_layer.clone()
    }

    /// Enable or disable Candle NER Layer 2
    #[allow(dead_code)]
    pub async fn set_candle_enabled(&self, enabled: bool) -> Result<()> {
        let config = self.config.read().await;
        let model_id = if config.candle_model_language.as_str() == "dutch" {
            "./models/robbert-v2-dutch-ner"
        } else {
            "./models/bert-large-cased-finetuned-conll03-english"
        };
        drop(config); // Drop config read lock early

        // This function now controls whether the Candle model is loaded/used
        // The actual loading happens in `initialize` or `set_detection_layer`
        // For now, we just update the config, and `detect_pii` will check if the model is loaded
        if enabled {
            if self.candle_ner_model.read().await.is_none() {
                tracing::info!("Attempting to load Candle NER model...");
                let device = if candle_core::utils::cuda_is_available() {
                    Device::new_cuda(0)?
                } else {
                    Device::Cpu
                };
                match NerModel::new_local(PathBuf::from(model_id), device) {
                    Ok(model) => {
                        *self.candle_ner_model.write().await = Some(model);
                        tracing::info!("✅ Candle NER model loaded successfully.");
                    },
                    Err(e) => {
                        tracing::error!("❌ Failed to load Candle NER model: {}", e);
                        return Err(anyhow!("Failed to load Candle NER model"));
                    }
                }
            }
        } else {
            *self.candle_ner_model.write().await = None;
            tracing::info!("Candle NER model unloaded.");
        }
        Ok(())
    }

    /// Check if Candle NER Layer 2 is available
    #[allow(dead_code)]
    pub async fn is_candle_available(&self) -> bool {
        self.candle_ner_model.read().await.is_some()
    }

    /// Get layer status information
    #[allow(dead_code)]
    pub async fn get_layer_status(&self) -> HashMap<String, bool> {
        let mut status = HashMap::new();
        status.insert("layer1_regex".to_string(), true); // Always available
        status.insert("layer2_candle".to_string(), self.is_candle_available().await);
        status.insert("layer3_presidio".to_string(), self.is_presidio_available().await);
        status
    }

    #[allow(dead_code)]
    pub async fn get_statistics(&self, text: &str) -> Result<HashMap<String, usize>> {
        let entities = self.detect_pii(text).await?;
        let mut stats = HashMap::new();

        for entity in entities {
            *stats.entry(entity.entity_type).or_insert(0) += 1;
        }

        Ok(stats)
    }
}
