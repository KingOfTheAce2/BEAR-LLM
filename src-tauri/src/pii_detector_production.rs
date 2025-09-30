use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::Command as AsyncCommand;
use std::path::PathBuf;

// Production PII Detector with Presidio integration and built-in fallback
// This is the single source of truth for PII detection in BEAR AI

lazy_static! {
    // Compiled regex patterns for performance
    static ref SSN_PATTERN: Regex = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap();
    static ref CREDIT_CARD_PATTERN: Regex = Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b").unwrap();
    static ref EMAIL_PATTERN: Regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    static ref PHONE_PATTERN: Regex = Regex::new(r"\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b").unwrap();
    static ref IP_PATTERN: Regex = Regex::new(r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b").unwrap();
    static ref CASE_NUMBER_PATTERN: Regex = Regex::new(r"\b(?:Case\s*(?:No\.?|Number)?:?\s*)?(\d{2,4}[-\s]?[A-Z]{2,4}[-\s]?\d{3,6})\b").unwrap();
    static ref MEDICAL_RECORD_PATTERN: Regex = Regex::new(r"\b(?:MRN|Medical Record(?:\s*Number)?):?\s*([A-Z0-9]{6,12})\b").unwrap();
    static ref NAME_PATTERN: Regex = Regex::new(r"\b([A-Z][a-z]+ (?:[A-Z]\. )?[A-Z][a-z]+)\b").unwrap();
    static ref TITLE_NAME_PATTERN: Regex = Regex::new(r"\b(?:Mr\.|Mrs\.|Ms\.|Dr\.|Prof\.|Judge|Attorney|Counselor)\s+([A-Z][a-z]+ ?[A-Z]?[a-z]*)\b").unwrap();
    static ref ORG_PATTERN: Regex = Regex::new(r"\b([A-Z][A-Za-z&\s]+ (?:Inc|LLC|LLP|Corp|Corporation|Company|Partners|Group|Associates|Firm|LTD|Limited))\b").unwrap();
    static ref LEGAL_ORG_PATTERN: Regex = Regex::new(r"\b(?:Law (?:Office|Firm) of |The )([A-Z][a-z]+ (?:& )?[A-Z][a-z]+)\b").unwrap();
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIIDetectionConfig {
    pub use_presidio: bool,
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
}

impl Default for PIIDetectionConfig {
    fn default() -> Self {
        Self {
            use_presidio: true,
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
        }
    }
}

pub struct PIIDetector {
    config: Arc<RwLock<PIIDetectionConfig>>,
    python_path: Arc<RwLock<Option<PathBuf>>>,
    presidio_available: Arc<RwLock<bool>>,
    custom_patterns: Arc<RwLock<HashMap<String, Regex>>>,
}

impl PIIDetector {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(PIIDetectionConfig::default())),
            python_path: Arc::new(RwLock::new(None)),
            presidio_available: Arc::new(RwLock::new(false)),
            custom_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Check for Python and Presidio
        self.check_presidio_availability().await;
        Ok(())
    }

    async fn check_presidio_availability(&self) {
        // Try to find Python
        for cmd in &["python3", "python", "py"] {
            if let Ok(output) = AsyncCommand::new(cmd)
                .arg("-c")
                .arg("import presidio_analyzer, presidio_anonymizer; print('OK')")
                .output()
                .await
            {
                if output.status.success() {
                    let mut python_path = self.python_path.write().await;
                    *python_path = Some(PathBuf::from(cmd));

                    let mut available = self.presidio_available.write().await;
                    *available = true;

                    println!("✅ Presidio is available for enhanced PII detection");
                    return;
                }
            }
        }

        println!("ℹ️ Presidio not available, using built-in PII detection");
    }

    pub async fn detect_pii(&self, text: &str) -> Result<Vec<PIIEntity>> {
        let config = self.config.read().await;
        let mut all_entities = Vec::new();

        // Phase 1: Try Presidio if available
        if config.use_presidio && *self.presidio_available.read().await {
            match self.detect_with_presidio(text).await {
                Ok(entities) => all_entities.extend(entities),
                Err(e) => eprintln!("Presidio detection error: {}", e),
            }
        }

        // Phase 2: Always run built-in detection for reliability
        all_entities.extend(self.detect_with_builtin(text, &config).await?);

        // Phase 3: Apply context enhancement if enabled
        if config.use_context_enhancement {
            all_entities = self.enhance_with_context(text, all_entities);
        }

        // Phase 4: Deduplicate and filter by confidence
        let filtered = self.deduplicate_and_filter(all_entities, config.confidence_threshold);

        Ok(filtered)
    }

    async fn detect_with_presidio(&self, text: &str) -> Result<Vec<PIIEntity>> {
        let python_path = self.python_path.read().await;
        let python = python_path.as_ref()
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

    async fn detect_with_builtin(&self, text: &str, config: &PIIDetectionConfig) -> Result<Vec<PIIEntity>> {
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
                    if context.contains("plaintiff") || context.contains("defendant") ||
                       context.contains("attorney") || context.contains("client") ||
                       context.contains("witness") || context.contains("judge") {
                        entity.confidence = (entity.confidence * 1.2).min(1.0);
                    }
                }
                "ORGANIZATION" => {
                    if context.contains("company") || context.contains("corporation") ||
                       context.contains("firm") || context.contains("agency") {
                        entity.confidence = (entity.confidence * 1.15).min(1.0);
                    }
                }
                "SSN" | "CREDIT_CARD" => {
                    if context.contains("social security") || context.contains("ssn") ||
                       context.contains("credit") || context.contains("card") {
                        entity.confidence = 1.0;
                    }
                }
                _ => {}
            }
        }

        entities
    }

    fn deduplicate_and_filter(&self, mut entities: Vec<PIIEntity>, threshold: f32) -> Vec<PIIEntity> {
        // Sort by position and confidence
        entities.sort_by(|a, b| {
            a.start.cmp(&b.start)
                .then(b.confidence.partial_cmp(&a.confidence).unwrap())
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
        const FALSE_POSITIVES: &[&str] = &[
            "United States", "New York", "Los Angeles", "San Francisco",
            "First Amendment", "Second Circuit", "Third Party", "Fourth Quarter",
            "Fifth Avenue", "Sixth Street", "Federal Court", "Supreme Court",
            "District Court", "Circuit Court"
        ];

        FALSE_POSITIVES.contains(&text)
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

    pub async fn add_custom_pattern(&self, name: String, pattern: String) -> Result<()> {
        let regex = Regex::new(&pattern)?;
        let mut patterns = self.custom_patterns.write().await;
        patterns.insert(name, regex);
        Ok(())
    }

    pub async fn update_config(&self, config: PIIDetectionConfig) -> Result<()> {
        let mut current = self.config.write().await;
        *current = config;
        Ok(())
    }

    pub async fn get_statistics(&self, text: &str) -> Result<HashMap<String, usize>> {
        let entities = self.detect_pii(text).await?;
        let mut stats = HashMap::new();

        for entity in entities {
            *stats.entry(entity.entity_type).or_insert(0) += 1;
        }

        Ok(stats)
    }
}