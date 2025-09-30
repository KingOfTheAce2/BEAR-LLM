//! # DEPRECATED - DO NOT USE
//!
//! This module is deprecated and maintained only for backward compatibility.
//! Use `pii_detector_production` instead for all new code.
//!
//! **Production module:** `pii_detector_production`
//!
//! While this v2 implementation has advanced features (NER models, custom recognizers),
//! the production version provides better real-world performance with Presidio integration
//! and built-in fallback mechanisms.
//!
//! **Migration path:** All code should use `pii_detector_production::PIIDetector`

#![deprecated(
    since = "1.0.5",
    note = "Use pii_detector_production instead. This module will be removed in v2.0.0"
)]

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use ort::{Environment, SessionBuilder, Value, tensor::OrtOwnedTensor};
use tokenizers::Tokenizer;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIIEntity {
    pub entity_type: String,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
    pub label: PIILabel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PIILabel {
    Person,
    Organization,
    Location,
    Email,
    Phone,
    SSN,
    CreditCard,
    IPAddress,
    MedicalRecord,
    CaseNumber,
    Date,
    Currency,
    Percentage,
    Custom(String),
}

impl PIILabel {
    pub fn to_string(&self) -> String {
        match self {
            PIILabel::Person => "PERSON",
            PIILabel::Organization => "ORG",
            PIILabel::Location => "LOC",
            PIILabel::Email => "EMAIL",
            PIILabel::Phone => "PHONE",
            PIILabel::SSN => "SSN",
            PIILabel::CreditCard => "CREDIT_CARD",
            PIILabel::IPAddress => "IP_ADDRESS",
            PIILabel::MedicalRecord => "MEDICAL_RECORD",
            PIILabel::CaseNumber => "CASE_NUMBER",
            PIILabel::Date => "DATE",
            PIILabel::Currency => "CURRENCY",
            PIILabel::Percentage => "PERCENTAGE",
            PIILabel::Custom(s) => s,
        }.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIIDetectionConfig {
    pub detect_names: bool,
    pub detect_organizations: bool,
    pub detect_locations: bool,
    pub detect_emails: bool,
    pub detect_phones: bool,
    pub detect_ssn: bool,
    pub detect_credit_cards: bool,
    pub detect_medical: bool,
    pub detect_legal: bool,
    pub confidence_threshold: f32,
    pub use_context: bool,
}

impl Default for PIIDetectionConfig {
    fn default() -> Self {
        Self {
            detect_names: true,
            detect_organizations: true,
            detect_locations: true,
            detect_emails: true,
            detect_phones: true,
            detect_ssn: true,
            detect_credit_cards: true,
            detect_medical: true,
            detect_legal: true,
            confidence_threshold: 0.85,
            use_context: true,
        }
    }
}

pub struct PIIDetectorV2 {
    config: Arc<RwLock<PIIDetectionConfig>>,
    ner_model: Arc<RwLock<Option<ort::Session>>>,
    tokenizer: Arc<RwLock<Option<Tokenizer>>>,
    regex_patterns: HashMap<String, Regex>,
    custom_recognizers: Arc<RwLock<Vec<CustomRecognizer>>>,
}

#[derive(Clone)]
struct CustomRecognizer {
    name: String,
    pattern: Regex,
    label: PIILabel,
    confidence: f32,
}

impl PIIDetectorV2 {
    pub fn new() -> Self {
        let mut regex_patterns = HashMap::new();

        // High-confidence regex patterns
        regex_patterns.insert(
            "ssn".to_string(),
            Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")
                .expect("CRITICAL: SSN regex pattern is invalid - this should never fail")
        );

        regex_patterns.insert(
            "credit_card".to_string(),
            Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b")
                .expect("CRITICAL: Credit card regex pattern is invalid - this should never fail")
        );

        regex_patterns.insert(
            "email".to_string(),
            Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
                .expect("CRITICAL: Email regex pattern is invalid - this should never fail")
        );

        regex_patterns.insert(
            "phone".to_string(),
            Regex::new(r"\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b")
                .expect("CRITICAL: Phone regex pattern is invalid - this should never fail")
        );

        regex_patterns.insert(
            "ip_address".to_string(),
            Regex::new(r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b")
                .expect("CRITICAL: IP address regex pattern is invalid - this should never fail")
        );

        // Legal-specific patterns
        regex_patterns.insert(
            "case_number".to_string(),
            Regex::new(r"\b(?:Case\s*(?:No\.?|Number)?:?\s*)?(\d{2,4}[-\s]?[A-Z]{2,4}[-\s]?\d{3,6})\b")
                .expect("CRITICAL: Case number regex pattern is invalid - this should never fail")
        );

        regex_patterns.insert(
            "medical_record".to_string(),
            Regex::new(r"\b(?:MRN|Medical Record(?:\s*Number)?):?\s*([A-Z0-9]{6,12})\b")
                .expect("CRITICAL: Medical record regex pattern is invalid - this should never fail")
        );

        Self {
            config: Arc::new(RwLock::new(PIIDetectionConfig::default())),
            ner_model: Arc::new(RwLock::new(None)),
            tokenizer: Arc::new(RwLock::new(None)),
            regex_patterns,
            custom_recognizers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Load NER model for entity recognition
        // In production, this would load a real ONNX model
        // For now, we'll prepare the structure for model loading

        // Model would be loaded like:
        // let environment = Environment::builder().build()?;
        // let model = SessionBuilder::new(&environment)?
        //     .with_model_from_file("models/ner_model.onnx")?;

        Ok(())
    }

    pub async fn detect_pii(&self, text: &str) -> Result<Vec<PIIEntity>> {
        let config = self.config.read().await;
        let mut entities = Vec::new();

        // Phase 1: Regex-based detection (high confidence)
        if config.detect_ssn {
            entities.extend(self.detect_with_regex(text, "ssn", PIILabel::SSN, 1.0));
        }

        if config.detect_credit_cards {
            entities.extend(self.detect_with_regex(text, "credit_card", PIILabel::CreditCard, 1.0));
        }

        if config.detect_emails {
            entities.extend(self.detect_with_regex(text, "email", PIILabel::Email, 1.0));
        }

        if config.detect_phones {
            entities.extend(self.detect_with_regex(text, "phone", PIILabel::Phone, 0.95));
        }

        if config.detect_legal {
            entities.extend(self.detect_with_regex(text, "case_number", PIILabel::CaseNumber, 0.9));
        }

        if config.detect_medical {
            entities.extend(self.detect_with_regex(text, "medical_record", PIILabel::MedicalRecord, 0.9));
        }

        // Phase 2: NER model-based detection
        let ner_entities = self.detect_with_ner(text).await?;

        // Filter by configured entity types and confidence
        for entity in ner_entities {
            if entity.confidence >= config.confidence_threshold {
                match &entity.label {
                    PIILabel::Person if config.detect_names => entities.push(entity),
                    PIILabel::Organization if config.detect_organizations => entities.push(entity),
                    PIILabel::Location if config.detect_locations => entities.push(entity),
                    _ => {}
                }
            }
        }

        // Phase 3: Custom recognizers
        let custom_recognizers = self.custom_recognizers.read().await;
        for recognizer in custom_recognizers.iter() {
            entities.extend(self.detect_with_custom_recognizer(text, recognizer));
        }

        // Phase 4: Context-aware post-processing
        if config.use_context {
            entities = self.apply_context_rules(text, entities).await;
        }

        // Deduplicate overlapping entities
        entities = self.deduplicate_entities(entities);

        Ok(entities)
    }

    fn detect_with_regex(&self, text: &str, pattern_key: &str, label: PIILabel, confidence: f32) -> Vec<PIIEntity> {
        let mut entities = Vec::new();

        if let Some(regex) = self.regex_patterns.get(pattern_key) {
            for capture in regex.find_iter(text) {
                entities.push(PIIEntity {
                    entity_type: label.to_string(),
                    text: capture.as_str().to_string(),
                    start: capture.start(),
                    end: capture.end(),
                    confidence,
                    label: label.clone(),
                });
            }
        }

        entities
    }

    async fn detect_with_ner(&self, text: &str) -> Result<Vec<PIIEntity>> {
        // This would use the actual NER model for inference
        // For now, we'll use advanced heuristics as a placeholder

        let mut entities = Vec::new();

        // Detect names using capitalization patterns and context
        entities.extend(self.detect_names_advanced(text));

        // Detect organizations using patterns and keywords
        entities.extend(self.detect_organizations_advanced(text));

        // Detect locations using gazetteers and patterns
        entities.extend(self.detect_locations_advanced(text));

        Ok(entities)
    }

    fn detect_names_advanced(&self, text: &str) -> Vec<PIIEntity> {
        let mut entities = Vec::new();

        // Common name patterns
        let name_pattern = Regex::new(r"\b([A-Z][a-z]+ (?:[A-Z]\. )?[A-Z][a-z]+)\b")
            .expect("CRITICAL: Name pattern regex is invalid - this should never fail");

        // Title patterns that often precede names
        let title_pattern = Regex::new(r"\b(?:Mr\.|Mrs\.|Ms\.|Dr\.|Prof\.|Judge|Attorney|Counselor)\s+([A-Z][a-z]+ ?[A-Z]?[a-z]*)\b")
            .expect("CRITICAL: Title pattern regex is invalid - this should never fail");

        for capture in name_pattern.find_iter(text) {
            let name = capture.as_str();

            // Apply context rules to filter out false positives
            if !self.is_likely_not_name(name) {
                entities.push(PIIEntity {
                    entity_type: "PERSON".to_string(),
                    text: name.to_string(),
                    start: capture.start(),
                    end: capture.end(),
                    confidence: 0.75,
                    label: PIILabel::Person,
                });
            }
        }

        for capture in title_pattern.captures_iter(text) {
            if let Some(name_match) = capture.get(1) {
                entities.push(PIIEntity {
                    entity_type: "PERSON".to_string(),
                    text: name_match.as_str().to_string(),
                    start: name_match.start(),
                    end: name_match.end(),
                    confidence: 0.9,
                    label: PIILabel::Person,
                });
            }
        }

        entities
    }

    fn detect_organizations_advanced(&self, text: &str) -> Vec<PIIEntity> {
        let mut entities = Vec::new();

        // Organization patterns
        let org_suffixes = vec!["Inc", "LLC", "LLP", "Corp", "Corporation", "Company", "Partners", "Group", "Associates", "Firm", "LTD", "Limited"];
        let pattern_str = format!(r"\b([A-Z][A-Za-z&\s]+ (?:{})\b)", org_suffixes.join("|"));
        let org_pattern = Regex::new(&pattern_str)
            .expect("CRITICAL: Organization pattern regex is invalid - this should never fail");

        // Legal entity patterns
        let legal_pattern = Regex::new(r"\b(?:Law (?:Office|Firm) of |The )([A-Z][a-z]+ (?:& )?[A-Z][a-z]+)\b")
            .expect("CRITICAL: Legal pattern regex is invalid - this should never fail");

        for capture in org_pattern.find_iter(text) {
            entities.push(PIIEntity {
                entity_type: "ORG".to_string(),
                text: capture.as_str().to_string(),
                start: capture.start(),
                end: capture.end(),
                confidence: 0.85,
                label: PIILabel::Organization,
            });
        }

        for capture in legal_pattern.captures_iter(text) {
            if let Some(org_match) = capture.get(1) {
                entities.push(PIIEntity {
                    entity_type: "ORG".to_string(),
                    text: org_match.as_str().to_string(),
                    start: org_match.start(),
                    end: org_match.end(),
                    confidence: 0.9,
                    label: PIILabel::Organization,
                });
            }
        }

        entities
    }

    fn detect_locations_advanced(&self, text: &str) -> Vec<PIIEntity> {
        let mut entities = Vec::new();

        // US state patterns
        let state_pattern = Regex::new(r"\b(Alabama|Alaska|Arizona|Arkansas|California|Colorado|Connecticut|Delaware|Florida|Georgia|Hawaii|Idaho|Illinois|Indiana|Iowa|Kansas|Kentucky|Louisiana|Maine|Maryland|Massachusetts|Michigan|Minnesota|Mississippi|Missouri|Montana|Nebraska|Nevada|New Hampshire|New Jersey|New Mexico|New York|North Carolina|North Dakota|Ohio|Oklahoma|Oregon|Pennsylvania|Rhode Island|South Carolina|South Dakota|Tennessee|Texas|Utah|Vermont|Virginia|Washington|West Virginia|Wisconsin|Wyoming|AL|AK|AZ|AR|CA|CO|CT|DE|FL|GA|HI|ID|IL|IN|IA|KS|KY|LA|ME|MD|MA|MI|MN|MS|MO|MT|NE|NV|NH|NJ|NM|NY|NC|ND|OH|OK|OR|PA|RI|SC|SD|TN|TX|UT|VT|VA|WA|WV|WI|WY)\b")
            .expect("CRITICAL: State pattern regex is invalid - this should never fail");

        // City patterns (simplified)
        let city_pattern = Regex::new(r"\b([A-Z][a-z]+(?: [A-Z][a-z]+)*), ([A-Z]{2})\b")
            .expect("CRITICAL: City pattern regex is invalid - this should never fail");

        for capture in state_pattern.find_iter(text) {
            entities.push(PIIEntity {
                entity_type: "LOC".to_string(),
                text: capture.as_str().to_string(),
                start: capture.start(),
                end: capture.end(),
                confidence: 0.95,
                label: PIILabel::Location,
            });
        }

        for capture in city_pattern.captures_iter(text) {
            if let Some(city_match) = capture.get(1) {
                entities.push(PIIEntity {
                    entity_type: "LOC".to_string(),
                    text: city_match.as_str().to_string(),
                    start: city_match.start(),
                    end: city_match.end(),
                    confidence: 0.8,
                    label: PIILabel::Location,
                });
            }
        }

        entities
    }

    fn detect_with_custom_recognizer(&self, text: &str, recognizer: &CustomRecognizer) -> Vec<PIIEntity> {
        let mut entities = Vec::new();

        for capture in recognizer.pattern.find_iter(text) {
            entities.push(PIIEntity {
                entity_type: recognizer.label.to_string(),
                text: capture.as_str().to_string(),
                start: capture.start(),
                end: capture.end(),
                confidence: recognizer.confidence,
                label: recognizer.label.clone(),
            });
        }

        entities
    }

    async fn apply_context_rules(&self, text: &str, mut entities: Vec<PIIEntity>) -> Vec<PIIEntity> {
        // Apply context-based rules to improve accuracy

        // Rule 1: If "John Doe" appears in legal context, it might be a placeholder
        entities.retain(|e| {
            if e.label == PIILabel::Person && (e.text == "John Doe" || e.text == "Jane Doe") {
                // Check if it's in a template context
                !text.contains("example") && !text.contains("template")
            } else {
                true
            }
        });

        // Rule 2: Boost confidence for entities near keywords
        for entity in &mut entities {
            let context_start = entity.start.saturating_sub(50);
            let context_end = (entity.end + 50).min(text.len());
            let context = &text[context_start..context_end];

            match entity.label {
                PIILabel::Person => {
                    if context.contains("plaintiff") || context.contains("defendant") ||
                       context.contains("attorney") || context.contains("client") {
                        entity.confidence = (entity.confidence * 1.2).min(1.0);
                    }
                }
                PIILabel::Organization => {
                    if context.contains("company") || context.contains("corporation") ||
                       context.contains("firm") {
                        entity.confidence = (entity.confidence * 1.15).min(1.0);
                    }
                }
                _ => {}
            }
        }

        entities
    }

    fn deduplicate_entities(&self, mut entities: Vec<PIIEntity>) -> Vec<PIIEntity> {
        // Sort by start position
        entities.sort_by_key(|e| e.start);

        let mut deduped = Vec::new();
        let mut last_end = 0;

        for entity in entities {
            // If entities overlap, keep the one with higher confidence
            if entity.start >= last_end {
                last_end = entity.end;
                deduped.push(entity);
            } else if !deduped.is_empty() {
                let last_idx = deduped.len() - 1;
                if entity.confidence > deduped[last_idx].confidence {
                    deduped[last_idx] = entity.clone();
                    last_end = entity.end;
                }
            }
        }

        deduped
    }

    fn is_likely_not_name(&self, text: &str) -> bool {
        // Common false positives for names
        let false_positives = vec![
            "United States", "New York", "Los Angeles", "First Amendment",
            "Second Circuit", "Third Party", "Fourth Quarter"
        ];

        false_positives.contains(&text)
    }

    pub async fn redact_pii(&self, text: &str) -> Result<String> {
        let entities = self.detect_pii(text).await?;
        let mut result = text.to_string();

        // Sort entities by position (reverse order for replacement)
        let mut sorted_entities = entities;
        sorted_entities.sort_by_key(|e| std::cmp::Reverse(e.start));

        for entity in sorted_entities {
            let replacement = format!("[REDACTED_{}]", entity.label.to_string());
            result.replace_range(entity.start..entity.end, &replacement);
        }

        Ok(result)
    }

    pub async fn anonymize_pii(&self, text: &str) -> Result<(String, HashMap<String, String>)> {
        let entities = self.detect_pii(text).await?;
        let mut result = text.to_string();
        let mut mappings = HashMap::new();

        // Sort entities by position (reverse order for replacement)
        let mut sorted_entities = entities;
        sorted_entities.sort_by_key(|e| std::cmp::Reverse(e.start));

        let mut counters: HashMap<String, usize> = HashMap::new();

        for entity in sorted_entities {
            let entity_type = entity.label.to_string();
            let counter = counters.entry(entity_type.clone()).or_insert(0);
            *counter += 1;

            let placeholder = format!("[{}_{:03}]", entity_type, counter);
            mappings.insert(placeholder.clone(), entity.text.clone());
            result.replace_range(entity.start..entity.end, &placeholder);
        }

        Ok((result, mappings))
    }

    pub async fn add_custom_recognizer(&self, name: String, pattern: String, label: PIILabel, confidence: f32) -> Result<()> {
        let regex = Regex::new(&pattern)?;
        let recognizer = CustomRecognizer {
            name,
            pattern: regex,
            label,
            confidence,
        };

        let mut recognizers = self.custom_recognizers.write().await;
        recognizers.push(recognizer);

        Ok(())
    }

    pub async fn update_config(&self, config: PIIDetectionConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    pub async fn get_statistics(&self, text: &str) -> Result<HashMap<String, usize>> {
        let entities = self.detect_pii(text).await?;
        let mut stats = HashMap::new();

        for entity in entities {
            *stats.entry(entity.label.to_string()).or_insert(0) += 1;
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_ssn() {
        let detector = PIIDetectorV2::new();
        let text = "My SSN is 123-45-6789 and that's private.";
        let entities = detector.detect_pii(text).await.unwrap();

        assert!(entities.iter().any(|e| e.label == PIILabel::SSN));
        assert!(entities.iter().any(|e| e.text == "123-45-6789"));
    }

    #[tokio::test]
    async fn test_detect_email() {
        let detector = PIIDetectorV2::new();
        let text = "Contact me at john.doe@example.com for details.";
        let entities = detector.detect_pii(text).await.unwrap();

        assert!(entities.iter().any(|e| e.label == PIILabel::Email));
        assert!(entities.iter().any(|e| e.text == "john.doe@example.com"));
    }

    #[tokio::test]
    async fn test_detect_names() {
        let detector = PIIDetectorV2::new();
        let text = "Attorney John Smith represented defendant Jane Doe.";
        let entities = detector.detect_pii(text).await.unwrap();

        assert!(entities.iter().any(|e| e.text == "John Smith" && e.label == PIILabel::Person));
    }

    #[tokio::test]
    async fn test_detect_organizations() {
        let detector = PIIDetectorV2::new();
        let text = "Smith & Associates LLC filed against Acme Corporation.";
        let entities = detector.detect_pii(text).await.unwrap();

        assert!(entities.iter().any(|e| e.label == PIILabel::Organization));
    }

    #[tokio::test]
    async fn test_redaction() {
        let detector = PIIDetectorV2::new();
        let text = "John Smith (SSN: 123-45-6789) works at Acme Corp.";
        let redacted = detector.redact_pii(text).await.unwrap();

        assert!(redacted.contains("[REDACTED_"));
        assert!(!redacted.contains("John Smith"));
        assert!(!redacted.contains("123-45-6789"));
    }
}