/// AI Act Transparency Module
///
/// Implements AI Act Article 13 transparency requirements for AI systems.
/// Provides user-facing notices, disclaimers, and confidence indicators.

pub mod confidence;
pub mod notices;

// Model card fetching and transparency
pub mod model_card_fetcher;
pub mod model_registry;
pub mod model_card_parser;
pub mod disclaimer_generator;
pub mod generic_disclaimer;

pub use model_card_fetcher::ModelCardFetcher;
pub use model_registry::ModelRegistry;
pub use model_card_parser::ModelCardParser;
pub use disclaimer_generator::{ModelDisclaimer, DisclaimerGenerator};
pub use generic_disclaimer::{GenericDisclaimer, GenericDisclaimerGenerator};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents the transparency status of an AI interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyContext {
    /// Unique identifier for this interaction
    pub interaction_id: String,

    /// Timestamp of the interaction
    pub timestamp: DateTime<Utc>,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Whether this is AI-generated content
    pub ai_generated: bool,

    /// Model name or identifier
    pub model_name: String,

    /// Whether human oversight is required
    pub requires_human_oversight: bool,

    /// Risk level of this interaction
    pub risk_level: RiskLevel,

    /// Whether user has acknowledged disclaimers
    pub disclaimers_acknowledged: bool,
}

/// Risk levels as defined by EU AI Act
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    /// Minimal risk - basic AI applications
    Minimal,

    /// Limited risk - requires transparency obligations
    Limited,

    /// High risk - legal advice, significant decisions
    High,

    /// Unacceptable risk - prohibited uses
    Unacceptable,
}

impl RiskLevel {
    /// Determine risk level based on context
    pub fn from_context(is_legal_advice: bool, affects_rights: bool) -> Self {
        if is_legal_advice && affects_rights {
            RiskLevel::High
        } else if is_legal_advice || affects_rights {
            RiskLevel::Limited
        } else {
            RiskLevel::Minimal
        }
    }

    /// Check if human oversight is required
    pub fn requires_human_oversight(&self) -> bool {
        matches!(self, RiskLevel::High | RiskLevel::Unacceptable)
    }

    /// Get warning message for this risk level
    pub fn warning_message(&self) -> &'static str {
        match self {
            RiskLevel::Minimal => "This is AI-generated content for informational purposes.",
            RiskLevel::Limited => "This AI-generated content should be reviewed before use.",
            RiskLevel::High => "⚠️ This AI-generated content requires professional review before use in legal contexts.",
            RiskLevel::Unacceptable => "❌ This operation is not permitted by AI regulations.",
        }
    }
}

impl TransparencyContext {
    /// Create a new transparency context for an AI interaction
    pub fn new(model_name: impl Into<String>, risk_level: RiskLevel) -> Self {
        Self {
            interaction_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            confidence: 0.0,
            ai_generated: true,
            model_name: model_name.into(),
            requires_human_oversight: risk_level.requires_human_oversight(),
            risk_level,
            disclaimers_acknowledged: false,
        }
    }

    /// Update confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Mark disclaimers as acknowledged
    pub fn acknowledge_disclaimers(mut self) -> Self {
        self.disclaimers_acknowledged = true;
        self
    }

    /// Get formatted transparency notice
    pub fn get_notice(&self) -> String {
        let mut notice = String::new();

        // AI disclosure
        notice.push_str("🤖 AI-Generated Content\n\n");

        // Model information
        notice.push_str(&format!("Model: {}\n", self.model_name));

        // Confidence indicator
        if self.confidence > 0.0 {
            notice.push_str(&format!(
                "Confidence: {}\n",
                self.get_confidence_indicator()
            ));
        }

        // Risk warning
        notice.push_str(&format!("\n{}\n", self.risk_level.warning_message()));

        // Human oversight requirement
        if self.requires_human_oversight {
            notice.push_str("\n⚖️ Professional review required before use in legal matters.\n");
        }

        // Timestamp
        notice.push_str(&format!("\nGenerated: {}\n", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));

        notice
    }

    /// Get visual confidence indicator
    fn get_confidence_indicator(&self) -> String {
        let bars = (self.confidence * 5.0).round() as usize;
        let filled = "█".repeat(bars);
        let empty = "░".repeat(5 - bars);
        format!("{}{} {:.0}%", filled, empty, self.confidence * 100.0)
    }
}

/// User preference for transparency notices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyPreferences {
    /// Show startup disclaimer
    pub show_startup_disclaimer: bool,

    /// Show per-response notices
    pub show_response_notices: bool,

    /// Show confidence indicators
    pub show_confidence: bool,

    /// Minimum confidence to show warnings
    pub min_confidence_warning: f32,

    /// Has user completed onboarding
    pub onboarding_completed: bool,

    /// Date of last disclaimer acknowledgment
    pub last_acknowledgment: Option<DateTime<Utc>>,
}

impl Default for TransparencyPreferences {
    fn default() -> Self {
        Self {
            show_startup_disclaimer: true,
            show_response_notices: true,
            show_confidence: true,
            min_confidence_warning: 0.7,
            onboarding_completed: false,
            last_acknowledgment: None,
        }
    }
}

impl TransparencyPreferences {
    /// Check if disclaimer needs to be shown again
    pub fn needs_disclaimer(&self) -> bool {
        if !self.onboarding_completed {
            return true;
        }

        if let Some(last_ack) = self.last_acknowledgment {
            // Show disclaimer again after 30 days
            let days_since = Utc::now().signed_duration_since(last_ack).num_days();
            days_since > 30
        } else {
            true
        }
    }

    /// Mark onboarding as completed
    pub fn complete_onboarding(mut self) -> Self {
        self.onboarding_completed = true;
        self.last_acknowledgment = Some(Utc::now());
        self
    }
}

#[cfg(test)]
mod transparency_tests {
    use super::*;

    #[test]
    fn test_risk_level_from_context() {
        assert_eq!(
            RiskLevel::from_context(true, true),
            RiskLevel::High
        );
        assert_eq!(
            RiskLevel::from_context(true, false),
            RiskLevel::Limited
        );
        assert_eq!(
            RiskLevel::from_context(false, false),
            RiskLevel::Minimal
        );
    }

    #[test]
    fn test_transparency_context_creation() {
        let ctx = TransparencyContext::new("test-model", RiskLevel::High)
            .with_confidence(0.85);

        assert_eq!(ctx.model_name, "test-model");
        assert_eq!(ctx.risk_level, RiskLevel::High);
        assert_eq!(ctx.confidence, 0.85);
        assert!(ctx.requires_human_oversight);
    }

    #[test]
    fn test_confidence_clamping() {
        let ctx = TransparencyContext::new("test", RiskLevel::Minimal)
            .with_confidence(1.5);

        assert_eq!(ctx.confidence, 1.0);

        let ctx2 = TransparencyContext::new("test", RiskLevel::Minimal)
            .with_confidence(-0.5);

        assert_eq!(ctx2.confidence, 0.0);
    }

    #[test]
    fn test_preferences_needs_disclaimer() {
        let prefs = TransparencyPreferences::default();
        assert!(prefs.needs_disclaimer());

        let completed = prefs.complete_onboarding();
        assert!(!completed.needs_disclaimer());
    }
}
