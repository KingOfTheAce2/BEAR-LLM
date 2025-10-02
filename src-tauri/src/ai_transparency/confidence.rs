/// AI Response Confidence Scoring
///
/// Implements confidence estimation for AI-generated responses
/// to help users assess reliability of outputs.
use serde::{Deserialize, Serialize};

/// Confidence score for an AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    /// Overall confidence (0.0 - 1.0)
    pub overall: f32,

    /// Individual confidence factors
    pub factors: ConfidenceFactors,

    /// Confidence level category
    pub level: ConfidenceLevel,

    /// Explanation of confidence score
    pub explanation: String,
}

/// Factors contributing to confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceFactors {
    /// Response completeness (0.0 - 1.0)
    pub completeness: f32,

    /// Context understanding (0.0 - 1.0)
    pub context_understanding: f32,

    /// Factual consistency (0.0 - 1.0)
    pub factual_consistency: f32,

    /// Response coherence (0.0 - 1.0)
    pub coherence: f32,

    /// Source reliability (0.0 - 1.0)
    pub source_reliability: f32,
}

/// Confidence level categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfidenceLevel {
    /// Very High confidence (>= 0.9)
    VeryHigh,

    /// High confidence (0.75 - 0.89)
    High,

    /// Medium confidence (0.5 - 0.74)
    Medium,

    /// Low confidence (0.25 - 0.49)
    Low,

    /// Very Low confidence (< 0.25)
    VeryLow,
}

impl ConfidenceLevel {
    /// Convert confidence score to level
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.9 => ConfidenceLevel::VeryHigh,
            s if s >= 0.75 => ConfidenceLevel::High,
            s if s >= 0.5 => ConfidenceLevel::Medium,
            s if s >= 0.25 => ConfidenceLevel::Low,
            _ => ConfidenceLevel::VeryLow,
        }
    }

    /// Get visual indicator
    pub fn indicator(&self) -> &'static str {
        match self {
            ConfidenceLevel::VeryHigh => "🟢 Very High",
            ConfidenceLevel::High => "🟢 High",
            ConfidenceLevel::Medium => "🟡 Medium",
            ConfidenceLevel::Low => "🟠 Low",
            ConfidenceLevel::VeryLow => "🔴 Very Low",
        }
    }

    /// Get recommendation text
    pub fn recommendation(&self) -> &'static str {
        match self {
            ConfidenceLevel::VeryHigh => {
                "Response appears highly reliable, but still verify important details."
            }
            ConfidenceLevel::High => "Response appears reliable, but verify critical information.",
            ConfidenceLevel::Medium => {
                "Response may be partially accurate. Verify all important details."
            }
            ConfidenceLevel::Low => {
                "Response has significant uncertainty. Thorough verification required."
            }
            ConfidenceLevel::VeryLow => {
                "Response has very low confidence. Use with extreme caution."
            }
        }
    }
}

impl ConfidenceScore {
    /// Create new confidence score from factors
    pub fn new(factors: ConfidenceFactors) -> Self {
        let overall = factors.calculate_overall();
        let level = ConfidenceLevel::from_score(overall);
        let explanation = Self::generate_explanation(&factors, level);

        Self {
            overall,
            factors,
            level,
            explanation,
        }
    }

    /// Generate explanation for the confidence score
    fn generate_explanation(factors: &ConfidenceFactors, level: ConfidenceLevel) -> String {
        let mut parts = Vec::new();

        // Identify strong factors
        if factors.completeness >= 0.8 {
            parts.push("complete response");
        }
        if factors.context_understanding >= 0.8 {
            parts.push("good context understanding");
        }
        if factors.factual_consistency >= 0.8 {
            parts.push("consistent facts");
        }
        if factors.coherence >= 0.8 {
            parts.push("coherent structure");
        }
        if factors.source_reliability >= 0.8 {
            parts.push("reliable sources");
        }

        // Identify weak factors
        let mut weaknesses = Vec::new();
        if factors.completeness < 0.5 {
            weaknesses.push("incomplete information");
        }
        if factors.context_understanding < 0.5 {
            weaknesses.push("limited context understanding");
        }
        if factors.factual_consistency < 0.5 {
            weaknesses.push("potential factual inconsistencies");
        }
        if factors.coherence < 0.5 {
            weaknesses.push("unclear structure");
        }
        if factors.source_reliability < 0.5 {
            weaknesses.push("uncertain sources");
        }

        let mut explanation = format!("Confidence level: {}. ", level.indicator());

        if !parts.is_empty() {
            explanation.push_str(&format!("Strengths: {}. ", parts.join(", ")));
        }

        if !weaknesses.is_empty() {
            explanation.push_str(&format!("Concerns: {}. ", weaknesses.join(", ")));
        }

        explanation.push_str(level.recommendation());

        explanation
    }

    /// Get formatted confidence display
    pub fn format_display(&self) -> String {
        format!(
            "{} ({:.0}%)\n{}\n\nFactors:\n• Completeness: {:.0}%\n• Context: {:.0}%\n• Consistency: {:.0}%\n• Coherence: {:.0}%\n• Sources: {:.0}%",
            self.level.indicator(),
            self.overall * 100.0,
            self.explanation,
            self.factors.completeness * 100.0,
            self.factors.context_understanding * 100.0,
            self.factors.factual_consistency * 100.0,
            self.factors.coherence * 100.0,
            self.factors.source_reliability * 100.0
        )
    }
}

impl ConfidenceFactors {
    /// Calculate overall confidence from individual factors
    pub fn calculate_overall(&self) -> f32 {
        // Weighted average of factors
        let weights = FactorWeights::default();

        (self.completeness * weights.completeness
            + self.context_understanding * weights.context_understanding
            + self.factual_consistency * weights.factual_consistency
            + self.coherence * weights.coherence
            + self.source_reliability * weights.source_reliability)
            / weights.total()
    }

    /// Create default factors with medium confidence
    pub fn default_medium() -> Self {
        Self {
            completeness: 0.6,
            context_understanding: 0.6,
            factual_consistency: 0.6,
            coherence: 0.6,
            source_reliability: 0.5,
        }
    }

    /// Create factors from response metadata
    pub fn from_response_metadata(
        token_count: usize,
        has_citations: bool,
        context_tokens: usize,
        response_coherence_score: Option<f32>,
    ) -> Self {
        // Estimate completeness based on response length
        let completeness = if token_count > 100 {
            0.8
        } else if token_count > 50 {
            0.6
        } else {
            0.4
        };

        // Estimate context understanding from context size
        let context_understanding = if context_tokens > 1000 {
            0.8
        } else if context_tokens > 500 {
            0.6
        } else {
            0.4
        };

        // Base factual consistency on citations
        let factual_consistency = if has_citations {
            0.7 // Still need verification
        } else {
            0.5
        };

        // Use provided coherence or estimate
        let coherence = response_coherence_score.unwrap_or(0.6);

        // Source reliability based on whether we have citations
        let source_reliability = if has_citations {
            0.6 // Citations need verification
        } else {
            0.4
        };

        Self {
            completeness,
            context_understanding,
            factual_consistency,
            coherence,
            source_reliability,
        }
    }
}

/// Weights for confidence factors
struct FactorWeights {
    completeness: f32,
    context_understanding: f32,
    factual_consistency: f32,
    coherence: f32,
    source_reliability: f32,
}

impl Default for FactorWeights {
    fn default() -> Self {
        Self {
            completeness: 1.0,
            context_understanding: 1.5, // More important
            factual_consistency: 2.0,   // Most important for legal
            coherence: 1.0,
            source_reliability: 1.5, // Very important
        }
    }
}

impl FactorWeights {
    fn total(&self) -> f32 {
        self.completeness
            + self.context_understanding
            + self.factual_consistency
            + self.coherence
            + self.source_reliability
    }
}

#[cfg(test)]
mod confidence_tests {
    use super::*;

    #[test]
    fn test_confidence_level_from_score() {
        assert_eq!(ConfidenceLevel::from_score(0.95), ConfidenceLevel::VeryHigh);
        assert_eq!(ConfidenceLevel::from_score(0.8), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.6), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.3), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.1), ConfidenceLevel::VeryLow);
    }

    #[test]
    fn test_confidence_score_creation() {
        let factors = ConfidenceFactors {
            completeness: 0.8,
            context_understanding: 0.7,
            factual_consistency: 0.9,
            coherence: 0.8,
            source_reliability: 0.7,
        };

        let score = ConfidenceScore::new(factors);
        assert!(score.overall > 0.7);
        assert!(score.overall < 0.9);
    }

    #[test]
    fn test_confidence_factors_from_metadata() {
        let factors = ConfidenceFactors::from_response_metadata(
            150,        // token_count
            true,       // has_citations
            800,        // context_tokens
            Some(0.75), // coherence
        );

        assert!(factors.completeness >= 0.6);
        assert!(factors.source_reliability > 0.4);
    }

    #[test]
    fn test_weighted_confidence_calculation() {
        let high_factual = ConfidenceFactors {
            completeness: 0.5,
            context_understanding: 0.5,
            factual_consistency: 0.9, // High factual consistency
            coherence: 0.5,
            source_reliability: 0.5,
        };

        let low_factual = ConfidenceFactors {
            completeness: 0.9,
            context_understanding: 0.9,
            factual_consistency: 0.3, // Low factual consistency
            coherence: 0.9,
            source_reliability: 0.9,
        };

        // Factual consistency should be weighted more heavily
        let high_score = high_factual.calculate_overall();
        let low_score = low_factual.calculate_overall();

        // Despite other factors being lower, high factual consistency should help
        assert!(high_score > 0.5);
        // Despite other factors being higher, low factual consistency should hurt
        assert!(low_score < 0.8);
    }
}
