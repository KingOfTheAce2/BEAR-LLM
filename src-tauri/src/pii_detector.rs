//! # DEPRECATED - DO NOT USE
//!
//! This module is deprecated and maintained only for backward compatibility.
//! Use `pii_detector_production` instead for all new code.
//!
//! **Production module:** `pii_detector_production`
//!
//! This legacy implementation lacks:
//! - Presidio integration
//! - Advanced context enhancement
//! - Credit card Luhn validation
//! - Production-grade error handling
//!
//! **Migration path:** All code should use `pii_detector_production::PIIDetector`

#![deprecated(
    since = "1.0.5",
    note = "Use pii_detector_production instead. This module will be removed in v2.0.0"
)]

use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;
use anyhow::Result;

lazy_static! {
    static ref SSN_REGEX: Regex = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b|\b\d{9}\b")
        .expect("CRITICAL: SSN regex pattern is invalid - this should never fail");
    static ref EMAIL_REGEX: Regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
        .expect("CRITICAL: Email regex pattern is invalid - this should never fail");
    static ref PHONE_REGEX: Regex = Regex::new(r"\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b")
        .expect("CRITICAL: Phone regex pattern is invalid - this should never fail");
    static ref CREDIT_CARD_REGEX: Regex = Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b")
        .expect("CRITICAL: Credit card regex pattern is invalid - this should never fail");
    static ref IP_REGEX: Regex = Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b")
        .expect("CRITICAL: IP address regex pattern is invalid - this should never fail");
    static ref DATE_OF_BIRTH_REGEX: Regex = Regex::new(r"\b(?:0[1-9]|1[0-2])[/\-](?:0[1-9]|[12]\d|3[01])[/\-](?:19|20)\d{2}\b")
        .expect("CRITICAL: Date of birth regex pattern is invalid - this should never fail");
    static ref PASSPORT_REGEX: Regex = Regex::new(r"\b[A-Z]{1,2}\d{6,9}\b")
        .expect("CRITICAL: Passport regex pattern is invalid - this should never fail");
    static ref DRIVER_LICENSE_REGEX: Regex = Regex::new(r"\b[A-Z]\d{7,12}\b")
        .expect("CRITICAL: Driver license regex pattern is invalid - this should never fail");
    static ref BANK_ACCOUNT_REGEX: Regex = Regex::new(r"\b\d{8,17}\b")
        .expect("CRITICAL: Bank account regex pattern is invalid - this should never fail");
    static ref ADDRESS_REGEX: Regex = Regex::new(r"\b\d+\s+[\w\s]+(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Lane|Ln|Drive|Dr|Court|Ct|Circle|Cir|Plaza|Pl|Way|Parkway|Pkwy)\b")
        .expect("CRITICAL: Address regex pattern is invalid - this should never fail");
    static ref CASE_NUMBER_REGEX: Regex = Regex::new(r"\b(?:Case|Docket|Matter)\s*(?:No\.?|Number|#)?\s*:?\s*[A-Z0-9\-]+\b")
        .expect("CRITICAL: Case number regex pattern is invalid - this should never fail");
    static ref EIN_REGEX: Regex = Regex::new(r"\b\d{2}-\d{7}\b")
        .expect("CRITICAL: EIN regex pattern is invalid - this should never fail");
    static ref MEDICAL_RECORD_REGEX: Regex = Regex::new(r"\b(?:MRN|Medical Record Number)\s*:?\s*[A-Z0-9]+\b")
        .expect("CRITICAL: Medical record regex pattern is invalid - this should never fail");
}

#[allow(dead_code)]
pub struct PIIDetector {
    custom_patterns: HashMap<String, Regex>,
    replacement_map: HashMap<String, String>,
    entity_counter: std::sync::atomic::AtomicUsize,
}

#[allow(dead_code)]
impl PIIDetector {
    pub fn new() -> Self {
        Self {
            custom_patterns: HashMap::new(),
            replacement_map: HashMap::new(),
            entity_counter: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub async fn remove_pii(&self, text: &str) -> Result<String> {
        let mut cleaned = text.to_string();
        let mut replacements = Vec::new();

        let patterns = vec![
            (&*SSN_REGEX, "SSN"),
            (&*EMAIL_REGEX, "EMAIL"),
            (&*PHONE_REGEX, "PHONE"),
            (&*CREDIT_CARD_REGEX, "CREDIT_CARD"),
            (&*IP_REGEX, "IP_ADDRESS"),
            (&*DATE_OF_BIRTH_REGEX, "DOB"),
            (&*PASSPORT_REGEX, "PASSPORT"),
            (&*DRIVER_LICENSE_REGEX, "DRIVER_LICENSE"),
            (&*BANK_ACCOUNT_REGEX, "BANK_ACCOUNT"),
            (&*ADDRESS_REGEX, "ADDRESS"),
            (&*CASE_NUMBER_REGEX, "CASE_NUMBER"),
            (&*EIN_REGEX, "EIN"),
            (&*MEDICAL_RECORD_REGEX, "MEDICAL_RECORD"),
        ];

        for (regex, pii_type) in patterns {
            for mat in regex.find_iter(text) {
                let id = self.entity_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let replacement = format!("[{}_REDACTED_{}]", pii_type, id);
                replacements.push((mat.start(), mat.end(), replacement));
            }
        }

        replacements.sort_by_key(|r| r.0);
        replacements.reverse();

        for (start, end, replacement) in replacements {
            cleaned.replace_range(start..end, &replacement);
        }

        cleaned = self.remove_names(&cleaned).await?;
        cleaned = self.remove_organizations(&cleaned).await?;

        Ok(cleaned)
    }

    async fn remove_names(&self, text: &str) -> Result<String> {
        let common_titles = vec![
            "Mr.", "Mrs.", "Ms.", "Miss", "Dr.", "Prof.", "Professor",
            "Judge", "Justice", "Attorney", "Counsel", "Esq.",
        ];

        let mut cleaned = text.to_string();
        for title in common_titles {
            let pattern = format!(r"\b{}\s+[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b", regex::escape(title));
            if let Ok(regex) = Regex::new(&pattern) {
                cleaned = regex.replace_all(&cleaned, "[NAME_REDACTED]").to_string();
            }
        }

        let name_pattern = Regex::new(r"\b[A-Z][a-z]+\s+(?:[A-Z]\.?\s+)?[A-Z][a-z]+\b")
            .expect("CRITICAL: Name pattern regex is invalid - this should never fail");
        cleaned = name_pattern.replace_all(&cleaned, |caps: &regex::Captures| {
            let text = caps.get(0).unwrap().as_str();
            if !self.is_common_phrase(text) {
                "[NAME_REDACTED]".to_string()
            } else {
                text.to_string()
            }
        }).to_string();

        Ok(cleaned)
    }

    async fn remove_organizations(&self, text: &str) -> Result<String> {
        let org_indicators = vec![
            "Inc.", "LLC", "LLP", "Ltd.", "Corp.", "Corporation",
            "Company", "Co.", "Partnership", "Associates", "Group",
            "Foundation", "Institute", "University", "College",
            "Hospital", "Clinic", "Bank", "Credit Union",
        ];

        let mut cleaned = text.to_string();
        for indicator in org_indicators {
            let pattern = format!(r"\b[\w\s]+\s+{}\b", regex::escape(indicator));
            if let Ok(regex) = Regex::new(&pattern) {
                cleaned = regex.replace_all(&cleaned, "[ORG_REDACTED]").to_string();
            }
        }

        Ok(cleaned)
    }

    fn is_common_phrase(&self, text: &str) -> bool {
        let common_phrases = vec![
            "United States", "New York", "Los Angeles", "Supreme Court",
            "District Court", "Circuit Court", "Court of Appeals",
            "Federal Government", "State Government", "Local Government",
        ];
        common_phrases.iter().any(|phrase| text.eq_ignore_ascii_case(phrase))
    }

    pub fn add_custom_pattern(&mut self, name: String, pattern: String) -> Result<()> {
        let regex = Regex::new(&pattern)?;
        self.custom_patterns.insert(name, regex);
        Ok(())
    }

    pub async fn detect_pii(&self, text: &str) -> Result<Vec<PIIMatch>> {
        let mut matches = Vec::new();

        let patterns = vec![
            (&*SSN_REGEX, "SSN"),
            (&*EMAIL_REGEX, "Email"),
            (&*PHONE_REGEX, "Phone"),
            (&*CREDIT_CARD_REGEX, "Credit Card"),
            (&*IP_REGEX, "IP Address"),
            (&*DATE_OF_BIRTH_REGEX, "Date of Birth"),
            (&*PASSPORT_REGEX, "Passport"),
            (&*DRIVER_LICENSE_REGEX, "Driver License"),
            (&*BANK_ACCOUNT_REGEX, "Bank Account"),
            (&*ADDRESS_REGEX, "Address"),
            (&*CASE_NUMBER_REGEX, "Case Number"),
            (&*EIN_REGEX, "EIN"),
            (&*MEDICAL_RECORD_REGEX, "Medical Record"),
        ];

        for (regex, pii_type) in patterns {
            for mat in regex.find_iter(text) {
                matches.push(PIIMatch {
                    pii_type: pii_type.to_string(),
                    start: mat.start(),
                    end: mat.end(),
                    text: mat.as_str().to_string(),
                });
            }
        }

        matches.sort_by_key(|m| m.start);
        Ok(matches)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PIIMatch {
    pub pii_type: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
}