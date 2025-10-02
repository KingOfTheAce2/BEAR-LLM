// SPDX-License-Identifier: MIT
// Copyright (c) 2025 BEAR AI LLM
//
// Risk Assessment Module for Model Transparency
// Provides model risk analysis with graceful degradation when HuggingFace is unavailable

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub model_id: String,
    pub risk_level: RiskLevel,
    pub disclaimers: Vec<Disclaimer>,
    pub limitations: Vec<String>,
    pub legal_suitability: LegalSuitability,
    pub data_source: DataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Unknown,
    LowRisk,
    MediumRisk,
    HighRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    HuggingFace,
    Cached,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disclaimer {
    pub category: DisclaimerCategory,
    pub text: String,
    pub severity: SeverityLevel,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisclaimerCategory {
    General,
    Legal,
    HighRisk,
    DataQuality,
    EUAIAct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeverityLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalSuitability {
    pub suitable_for_legal_work: bool,
    pub limitations: Vec<String>,
    pub recommended_safeguards: Vec<String>,
}

pub struct RiskAssessor {
    cache: Arc<RwLock<HashMap<String, (RiskAssessment, std::time::Instant)>>>,
    cache_duration: std::time::Duration,
}

impl RiskAssessor {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_duration: std::time::Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }

    /// Main entry point - attempts to get risk assessment with graceful fallback
    pub async fn assess_model(&self, model_id: &str) -> RiskAssessment {
        // Check cache first
        if let Some(cached) = self.get_cached(model_id).await {
            return cached;
        }

        // Try to fetch from Hugging Face
        match self.fetch_from_huggingface(model_id).await {
            Ok(assessment) => {
                self.cache_assessment(model_id, assessment.clone()).await;
                assessment
            }
            Err(e) => {
                tracing::warn!(
                    model_id = %model_id,
                    error = %e,
                    "Failed to fetch model info from Hugging Face, using fallback"
                );
                self.create_fallback_assessment(model_id)
            }
        }
    }

    async fn get_cached(&self, model_id: &str) -> Option<RiskAssessment> {
        let cache = self.cache.read().await;
        if let Some((assessment, timestamp)) = cache.get(model_id) {
            if timestamp.elapsed() < self.cache_duration {
                tracing::debug!(model_id = %model_id, "Using cached risk assessment");
                return Some(assessment.clone());
            }
        }
        None
    }

    async fn cache_assessment(&self, model_id: &str, assessment: RiskAssessment) {
        let mut cache = self.cache.write().await;
        cache.insert(model_id.to_string(), (assessment, std::time::Instant::now()));
    }

    /// Attempt to fetch model card from Hugging Face
    async fn fetch_from_huggingface(&self, model_id: &str) -> Result<RiskAssessment, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://huggingface.co/api/models/{}", model_id);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(format!("HF API returned status {}", response.status()).into());
        }

        let model_info: serde_json::Value = response.json().await?;

        // Parse the model card
        self.parse_model_info(model_id, &model_info).await
    }

    async fn parse_model_info(
        &self,
        model_id: &str,
        info: &serde_json::Value,
    ) -> Result<RiskAssessment, Box<dyn std::error::Error + Send + Sync>> {
        let mut limitations = Vec::new();
        let mut disclaimers = Vec::new();

        // Extract limitations from model card
        if let Some(card_data) = info.get("cardData") {
            if let Some(lims) = card_data.get("limitations") {
                if let Some(lim_text) = lims.as_str() {
                    limitations.push(lim_text.to_string());
                }
            }

            if let Some(bias) = card_data.get("bias") {
                if let Some(bias_text) = bias.as_str() {
                    limitations.push(format!("Potential biases: {}", bias_text));
                }
            }
        }

        // Check tags for risk indicators
        let tags = info.get("tags")
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Determine risk level based on model characteristics
        let risk_level = self.determine_risk_level(&tags, &limitations);

        // Always add mandatory legal disclaimers
        disclaimers.extend(self.create_legal_disclaimers());

        // Add risk-specific disclaimers
        if matches!(risk_level, RiskLevel::HighRisk | RiskLevel::MediumRisk) {
            disclaimers.extend(self.create_high_risk_disclaimers());
        }

        // Add EU AI Act disclaimer if applicable
        disclaimers.push(self.create_eu_ai_act_disclaimer());

        // Assess legal suitability
        let legal_suitability = self.assess_legal_suitability(&tags, &limitations);

        Ok(RiskAssessment {
            model_id: model_id.to_string(),
            risk_level,
            disclaimers,
            limitations,
            legal_suitability,
            data_source: DataSource::HuggingFace,
        })
    }

    fn determine_risk_level(&self, tags: &[&str], limitations: &[String]) -> RiskLevel {
        // Check for indicators of high-risk usage
        let high_risk_indicators = [
            "legal",
            "medical",
            "finance",
            "law",
            "court",
            "contract",
        ];

        let has_high_risk = tags.iter().any(|tag| {
            high_risk_indicators.iter().any(|indicator| {
                tag.to_lowercase().contains(indicator)
            })
        });

        let has_severe_limitations = limitations.iter().any(|lim| {
            lim.to_lowercase().contains("not suitable for production")
                || lim.to_lowercase().contains("research only")
                || lim.to_lowercase().contains("high error rate")
        });

        if has_high_risk || has_severe_limitations {
            RiskLevel::HighRisk
        } else if tags.contains(&"conversational") || tags.contains(&"text-generation") {
            RiskLevel::MediumRisk
        } else {
            RiskLevel::LowRisk
        }
    }

    fn assess_legal_suitability(&self, _tags: &[&str], limitations: &[String]) -> LegalSuitability {
        let suitable = !limitations.iter().any(|lim| {
            lim.to_lowercase().contains("not suitable for")
                || lim.to_lowercase().contains("research only")
        });

        let limitations = vec![
            "AI-generated output requires human review and verification".to_string(),
            "Cannot replace professional legal judgment".to_string(),
            "May produce factually incorrect or outdated information".to_string(),
            "Not trained specifically on your jurisdiction's laws".to_string(),
        ];

        let recommended_safeguards = vec![
            "Always verify legal citations and references".to_string(),
            "Have outputs reviewed by qualified legal professionals".to_string(),
            "Do not rely solely on AI for legal advice or decisions".to_string(),
            "Maintain human oversight for all legal work products".to_string(),
            "Keep audit trails of AI-assisted work".to_string(),
        ];

        LegalSuitability {
            suitable_for_legal_work: suitable,
            limitations,
            recommended_safeguards,
        }
    }

    fn create_legal_disclaimers(&self) -> Vec<Disclaimer> {
        vec![
            Disclaimer {
                category: DisclaimerCategory::Legal,
                text: "NOT LEGAL ADVICE: This AI system does not provide legal advice. Output should not be relied upon as a substitute for consultation with qualified legal professionals.".to_string(),
                severity: SeverityLevel::Critical,
                required: true,
            },
            Disclaimer {
                category: DisclaimerCategory::Legal,
                text: "NO ATTORNEY-CLIENT RELATIONSHIP: Use of this system does not create an attorney-client relationship.".to_string(),
                severity: SeverityLevel::Warning,
                required: true,
            },
            Disclaimer {
                category: DisclaimerCategory::Legal,
                text: "VERIFY ALL OUTPUT: All AI-generated content must be independently verified by qualified legal professionals before use.".to_string(),
                severity: SeverityLevel::Critical,
                required: true,
            },
        ]
    }

    fn create_high_risk_disclaimers(&self) -> Vec<Disclaimer> {
        vec![
            Disclaimer {
                category: DisclaimerCategory::HighRisk,
                text: "HIGH-RISK AI SYSTEM: This system may be classified as high-risk under EU AI Act regulations when used for legal work.".to_string(),
                severity: SeverityLevel::Warning,
                required: true,
            },
            Disclaimer {
                category: DisclaimerCategory::DataQuality,
                text: "KNOWN LIMITATIONS: AI models can produce incorrect, biased, or outdated information. Human oversight is mandatory.".to_string(),
                severity: SeverityLevel::Warning,
                required: true,
            },
        ]
    }

    fn create_eu_ai_act_disclaimer(&self) -> Disclaimer {
        Disclaimer {
            category: DisclaimerCategory::EUAIAct,
            text: "EU AI ACT COMPLIANCE: This application may be subject to EU AI Act regulations. Users are responsible for ensuring compliance with applicable laws in their jurisdiction.".to_string(),
            severity: SeverityLevel::Info,
            required: false,
        }
    }

    /// Fallback when Hugging Face is unavailable
    fn create_fallback_assessment(&self, model_id: &str) -> RiskAssessment {
        tracing::info!(
            model_id = %model_id,
            "Creating fallback risk assessment with conservative assumptions"
        );

        RiskAssessment {
            model_id: model_id.to_string(),
            risk_level: RiskLevel::Unknown,
            disclaimers: self.create_comprehensive_fallback_disclaimers(),
            limitations: vec![
                "Model information unavailable - unable to verify capabilities".to_string(),
                "Risk assessment based on conservative assumptions".to_string(),
                "Users should exercise additional caution".to_string(),
            ],
            legal_suitability: LegalSuitability {
                suitable_for_legal_work: false, // Conservative default
                limitations: vec![
                    "Model capabilities and limitations unknown".to_string(),
                    "Cannot verify training data or bias characteristics".to_string(),
                    "Suitability for legal work cannot be determined".to_string(),
                ],
                recommended_safeguards: vec![
                    "Exercise extreme caution when using for legal work".to_string(),
                    "Require additional human review and verification".to_string(),
                    "Consider using well-documented models with known characteristics".to_string(),
                ],
            },
            data_source: DataSource::Fallback,
        }
    }

    fn create_comprehensive_fallback_disclaimers(&self) -> Vec<Disclaimer> {
        let mut disclaimers = self.create_legal_disclaimers();
        disclaimers.extend(self.create_high_risk_disclaimers());
        disclaimers.push(self.create_eu_ai_act_disclaimer());

        // Add extra disclaimer about unknown model
        disclaimers.push(Disclaimer {
            category: DisclaimerCategory::General,
            text: "UNKNOWN MODEL CHARACTERISTICS: Unable to retrieve model information. This assessment uses conservative assumptions. Proceed with caution.".to_string(),
            severity: SeverityLevel::Warning,
            required: true,
        });

        disclaimers
    }
}

impl Default for RiskAssessor {
    fn default() -> Self {
        Self::new()
    }
}
