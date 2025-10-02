use super::model_card_parser::ModelCard;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDisclaimer {
    pub model_name: String,
    pub model_id: String,
    pub summary: String,
    pub warnings: Vec<String>,
    pub capabilities: Vec<String>,
    pub limitations: Vec<String>,
    pub recommended_use: String,
    pub not_recommended_use: String,
    pub acknowledgment_required: bool,
}

pub struct DisclaimerGenerator;

impl DisclaimerGenerator {
    /// Generate disclaimer from model card
    pub fn generate(model_card: &ModelCard) -> ModelDisclaimer {
        let model_name = Self::extract_model_name(&model_card.model_id);

        ModelDisclaimer {
            model_name: model_name.clone(),
            model_id: model_card.model_id.clone(),
            summary: Self::generate_summary(model_card),
            warnings: Self::generate_warnings(model_card),
            capabilities: Self::extract_capabilities(model_card),
            limitations: Self::consolidate_limitations(model_card),
            recommended_use: Self::generate_recommended_use(model_card),
            not_recommended_use: Self::generate_not_recommended_use(model_card),
            acknowledgment_required: Self::requires_acknowledgment(model_card),
        }
    }

    /// Extract clean model name from model ID
    fn extract_model_name(model_id: &str) -> String {
        // Extract name after last slash
        model_id
            .split('/')
            .next_back()
            .unwrap_or(model_id)
            .replace(['-', '_'], " ")
    }

    /// Generate human-readable summary
    fn generate_summary(model_card: &ModelCard) -> String {
        let mut summary = format!("Model: {}\n", model_card.model_id);

        if !model_card.description.is_empty() {
            summary.push('\n');
            summary.push_str(&model_card.description);
            summary.push('\n');
        }

        if let Some(ref license) = model_card.license {
            summary.push_str("\nLicense: ");
            summary.push_str(license);
            summary.push('\n');
        }

        summary
    }

    /// Generate warning messages
    fn generate_warnings(model_card: &ModelCard) -> Vec<String> {
        let mut warnings = Vec::new();

        // Add safety warnings from model card
        warnings.extend(model_card.safety_warnings.clone());

        // Standard AI warnings
        warnings.push("⚠️ Known Limitations:".to_string());

        // Add specific limitations
        if model_card.limitations.is_empty() {
            warnings.push(
                "• May produce inaccurate or misleading information (hallucinations)".to_string(),
            );
            warnings.push("• Not suitable for medical, legal, or financial advice".to_string());
            warnings.push("• May have knowledge cutoff date limitations".to_string());
        } else {
            for limitation in &model_card.limitations {
                warnings.push(format!("• {}", limitation));
            }
        }

        // Add bias warnings
        if !model_card.biases.is_empty() {
            warnings.push("\n⚠️ Known Biases:".to_string());
            for bias in &model_card.biases {
                warnings.push(format!("• {}", bias));
            }
        } else {
            warnings.push("\n⚠️ Potential Biases:".to_string());
            warnings.push("• May exhibit biases from training data".to_string());
            warnings.push("• May reflect societal stereotypes and prejudices".to_string());
        }

        warnings
    }

    /// Extract model capabilities
    fn extract_capabilities(model_card: &ModelCard) -> Vec<String> {
        let mut capabilities = Vec::new();

        // Use intended use as capabilities
        for use_case in &model_card.intended_use {
            capabilities.push(use_case.clone());
        }

        // If no specific use cases, add generic capabilities
        if capabilities.is_empty() {
            capabilities.push("General conversational AI".to_string());
            capabilities.push("Text generation and completion".to_string());
            capabilities.push("Question answering".to_string());
        }

        capabilities
    }

    /// Consolidate limitations from multiple sources
    fn consolidate_limitations(model_card: &ModelCard) -> Vec<String> {
        let mut limitations = Vec::new();

        // Add explicit limitations
        limitations.extend(model_card.limitations.clone());

        // Add biases as limitations
        for bias in &model_card.biases {
            limitations.push(format!("Bias: {}", bias));
        }

        // Add ethical considerations
        for consideration in &model_card.ethical_considerations {
            limitations.push(format!("Ethical consideration: {}", consideration));
        }

        // If no specific limitations, add generic ones
        if limitations.is_empty() {
            limitations.push("May generate false or misleading information".to_string());
            limitations.push("Limited to training data knowledge cutoff".to_string());
            limitations.push("Cannot verify real-time information".to_string());
            limitations.push("May produce biased or inappropriate content".to_string());
        }

        limitations
    }

    /// Generate recommended use statement
    fn generate_recommended_use(model_card: &ModelCard) -> String {
        if model_card.intended_use.is_empty() {
            return "Research, education, and general-purpose conversational AI applications where accuracy is not critical.".to_string();
        }

        model_card.intended_use.join(", ")
    }

    /// Generate not-recommended use statement
    fn generate_not_recommended_use(_model_card: &ModelCard) -> String {
        let restricted_uses = [
            "Medical diagnosis or treatment decisions",
            "Legal advice or court proceedings",
            "Financial investment decisions",
            "Critical safety systems",
            "Military or surveillance applications",
            "Automated decision-making affecting human rights",
        ];

        restricted_uses.join(", ")
    }

    /// Determine if user acknowledgment is required
    fn requires_acknowledgment(model_card: &ModelCard) -> bool {
        // Require acknowledgment if there are safety warnings or significant limitations
        !model_card.safety_warnings.is_empty()
            || model_card.limitations.len() > 3
            || !model_card.biases.is_empty()
    }

    /// Generate formatted disclaimer text for display
    pub fn format_for_display(disclaimer: &ModelDisclaimer) -> String {
        let mut text = String::new();

        text.push_str("═══════════════════════════════════════════\n");
        text.push_str("  AI MODEL DISCLOSURE\n");
        text.push_str("═══════════════════════════════════════════\n\n");

        text.push_str(&disclaimer.summary);
        text.push('\n');

        text.push_str("📋 Intended Use:\n");
        for capability in &disclaimer.capabilities {
            text.push_str("  • ");
            text.push_str(capability);
            text.push('\n');
        }
        text.push('\n');

        text.push_str("⚠️  IMPORTANT WARNINGS:\n");
        for warning in &disclaimer.warnings {
            text.push_str(warning);
            text.push('\n');
        }
        text.push('\n');

        text.push_str("🚫 NOT Recommended For:\n  ");
        text.push_str(&disclaimer.not_recommended_use);
        text.push_str("\n\n");

        text.push_str("═══════════════════════════════════════════\n");

        if disclaimer.acknowledgment_required {
            text.push_str("\n⚠️  Please acknowledge that you understand these\n");
            text.push_str("   limitations before proceeding.\n");
        }

        text
    }

    /// Generate minimal inline disclaimer
    pub fn format_inline(disclaimer: &ModelDisclaimer) -> String {
        format!(
            "⚠️ {} - {} | Limitations apply. Not for critical decisions.",
            disclaimer.model_name,
            disclaimer
                .capabilities
                .first()
                .unwrap_or(&"AI Model".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_disclaimer() {
        let model_card = ModelCard {
            model_id: "meta-llama/Llama-2-7b-chat-hf".to_string(),
            description: "Conversational AI model".to_string(),
            intended_use: vec!["Chat applications".to_string(), "Research".to_string()],
            limitations: vec!["May hallucinate".to_string()],
            biases: vec!["Training data biases".to_string()],
            training_data: Some("2T tokens".to_string()),
            license: Some("Llama 2 License".to_string()),
            paper_url: None,
            ethical_considerations: vec![],
            safety_warnings: vec!["Use with caution".to_string()],
            performance_metrics: vec![],
        };

        let disclaimer = DisclaimerGenerator::generate(&model_card);

        assert_eq!(disclaimer.model_id, "meta-llama/Llama-2-7b-chat-hf");
        assert!(!disclaimer.warnings.is_empty());
        assert!(!disclaimer.capabilities.is_empty());
        assert!(disclaimer.acknowledgment_required);
    }

    #[test]
    fn test_format_for_display() {
        let disclaimer = ModelDisclaimer {
            model_name: "Test Model".to_string(),
            model_id: "test/model".to_string(),
            summary: "A test model".to_string(),
            warnings: vec!["Warning 1".to_string()],
            capabilities: vec!["Capability 1".to_string()],
            limitations: vec!["Limitation 1".to_string()],
            recommended_use: "Testing".to_string(),
            not_recommended_use: "Production".to_string(),
            acknowledgment_required: true,
        };

        let display = DisclaimerGenerator::format_for_display(&disclaimer);
        assert!(display.contains("AI MODEL DISCLOSURE"));
        assert!(display.contains("Warning 1"));
    }

    #[test]
    fn test_format_inline() {
        let disclaimer = ModelDisclaimer {
            model_name: "Test Model".to_string(),
            model_id: "test/model".to_string(),
            summary: "".to_string(),
            warnings: vec![],
            capabilities: vec!["Testing".to_string()],
            limitations: vec![],
            recommended_use: "".to_string(),
            not_recommended_use: "".to_string(),
            acknowledgment_required: false,
        };

        let inline = DisclaimerGenerator::format_inline(&disclaimer);
        assert!(inline.contains("Test Model"));
        assert!(inline.contains("⚠️"));
    }
}
