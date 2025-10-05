use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericDisclaimer {
    pub title: String,
    pub warning_level: DisclaimerLevel,
    pub message: String,
    pub limitations: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisclaimerLevel {
    Info,
    Warning,
    Critical,
}

pub struct GenericDisclaimerGenerator;

impl GenericDisclaimerGenerator {
    /// Generate generic disclaimer for unknown models
    pub fn generate_unknown_model(model_name: &str) -> GenericDisclaimer {
        GenericDisclaimer {
            title: "⚠️ Model Information Unavailable".to_string(),
            warning_level: DisclaimerLevel::Warning,
            message: format!(
                "Information about model '{}' could not be retrieved.\n\
                Please exercise caution and verify all outputs.",
                model_name
            ),
            limitations: vec![
                "Model capabilities and limitations are unknown".to_string(),
                "May generate false or misleading information".to_string(),
                "Not a substitute for professional advice".to_string(),
                "Training data sources and biases are unknown".to_string(),
                "Cannot verify real-time information".to_string(),
                "No information about ethical considerations".to_string(),
            ],
            recommendations: vec![
                "Verify all important information from authoritative sources".to_string(),
                "Do not use for critical decision-making".to_string(),
                "Be aware of potential biases in outputs".to_string(),
                "Treat all outputs with appropriate skepticism".to_string(),
            ],
        }
    }

    /// Generate generic AI limitations disclaimer
    pub fn generate_general_ai_disclaimer() -> GenericDisclaimer {
        GenericDisclaimer {
            title: "⚠️ General AI System Disclaimer".to_string(),
            warning_level: DisclaimerLevel::Info,
            message: "This is an artificial intelligence system with inherent limitations."
                .to_string(),
            limitations: vec![
                "May generate inaccurate or false information (hallucinations)".to_string(),
                "Outputs reflect patterns in training data, which may contain biases".to_string(),
                "Cannot access real-time information or verify current facts".to_string(),
                "Not suitable for medical, legal, financial, or other professional advice"
                    .to_string(),
                "May produce inappropriate or harmful content despite safeguards".to_string(),
                "Cannot understand context or nuance like humans do".to_string(),
            ],
            recommendations: vec![
                "Always verify critical information from authoritative sources".to_string(),
                "Use human judgment for important decisions".to_string(),
                "Be aware that AI outputs may reflect societal biases".to_string(),
                "Do not share sensitive personal information".to_string(),
                "Consult qualified professionals for specialized advice".to_string(),
            ],
        }
    }

    /// Generate disclaimer for offline/cache failure
    pub fn generate_offline_disclaimer(model_name: &str) -> GenericDisclaimer {
        GenericDisclaimer {
            title: "⚠️ Operating in Offline Mode".to_string(),
            warning_level: DisclaimerLevel::Warning,
            message: format!(
                "Could not retrieve model card for '{}' due to network issues.\n\
                Using cached information or generic disclaimer.",
                model_name
            ),
            limitations: vec![
                "Model information may be outdated".to_string(),
                "Latest safety updates may not be available".to_string(),
                "Cannot verify current model capabilities".to_string(),
            ],
            recommendations: vec![
                "Connect to the internet for updated model information".to_string(),
                "Exercise extra caution when using models offline".to_string(),
                "Check for updates when connection is restored".to_string(),
            ],
        }
    }

    /// Generate disclaimer for high-risk applications
    pub fn generate_high_risk_disclaimer() -> GenericDisclaimer {
        GenericDisclaimer {
            title: "🚨 HIGH-RISK APPLICATION WARNING".to_string(),
            warning_level: DisclaimerLevel::Critical,
            message:
                "AI systems should NOT be used for high-risk applications without human oversight."
                    .to_string(),
            limitations: vec![
                "AI cannot be held legally accountable for decisions".to_string(),
                "May fail in unpredictable ways".to_string(),
                "Cannot guarantee safety in critical situations".to_string(),
                "Not validated for high-stakes decision-making".to_string(),
            ],
            recommendations: vec![
                "NEVER use for medical diagnosis or treatment without doctor consultation"
                    .to_string(),
                "NEVER use for legal advice without lawyer review".to_string(),
                "NEVER use for financial decisions without expert consultation".to_string(),
                "NEVER use for safety-critical systems without human oversight".to_string(),
                "NEVER use for automated decisions affecting human rights".to_string(),
            ],
        }
    }

    /// Generate EU AI Act compliance disclaimer
    pub fn generate_ai_act_disclaimer() -> GenericDisclaimer {
        GenericDisclaimer {
            title: "🇪🇺 EU AI Act Transparency Notice".to_string(),
            warning_level: DisclaimerLevel::Info,
            message: "You are interacting with an AI system. Under EU regulations, you have the right to be informed.".to_string(),
            limitations: vec![
                "This is an AI-generated interaction, not human communication".to_string(),
                "AI systems may produce incorrect or biased outputs".to_string(),
                "Outputs are based on statistical patterns, not understanding".to_string(),
            ],
            recommendations: vec![
                "You have the right to know when you're interacting with AI".to_string(),
                "You have the right to understand how AI systems affect you".to_string(),
                "You have the right to challenge automated decisions".to_string(),
                "You have the right to human review of important decisions".to_string(),
            ],
        }
    }

    /// Format disclaimer for display
    pub fn format_for_display(disclaimer: &GenericDisclaimer) -> String {
        let mut text = String::new();

        // Header
        text.push_str("═══════════════════════════════════════════\n");
        text.push_str(&format!("  {}\n", disclaimer.title));
        text.push_str("═══════════════════════════════════════════\n\n");

        // Message
        text.push_str(&disclaimer.message);
        text.push_str("\n\n");

        // Limitations
        if !disclaimer.limitations.is_empty() {
            text.push_str("⚠️  Limitations:\n");
            for limitation in &disclaimer.limitations {
                text.push_str(&format!("  • {}\n", limitation));
            }
            text.push('\n');
        }

        // Recommendations
        if !disclaimer.recommendations.is_empty() {
            text.push_str("✓  Recommendations:\n");
            for recommendation in &disclaimer.recommendations {
                text.push_str(&format!("  • {}\n", recommendation));
            }
            text.push('\n');
        }

        text.push_str("═══════════════════════════════════════════\n");

        text
    }

    /// Format minimal inline disclaimer
    #[allow(dead_code)]
    pub fn format_inline(disclaimer: &GenericDisclaimer) -> String {
        let icon = match disclaimer.warning_level {
            DisclaimerLevel::Info => "ℹ️",
            DisclaimerLevel::Warning => "⚠️",
            DisclaimerLevel::Critical => "🚨",
        };

        format!(
            "{} {} - Exercise caution with AI outputs",
            icon,
            disclaimer
                .title
                .trim_start_matches("⚠️ ")
                .trim_start_matches("🚨 ")
                .trim_start_matches("🇪🇺 ")
                .trim_start_matches("ℹ️ ")
        )
    }

    /// Get disclaimer severity level
    #[allow(dead_code)]
    pub fn get_severity_color(level: &DisclaimerLevel) -> &'static str {
        match level {
            DisclaimerLevel::Info => "#3B82F6",     // Blue
            DisclaimerLevel::Warning => "#F59E0B",  // Amber
            DisclaimerLevel::Critical => "#EF4444", // Red
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unknown_model() {
        let disclaimer = GenericDisclaimerGenerator::generate_unknown_model("test-model");

        assert!(disclaimer.title.contains("Unavailable"));
        assert!(!disclaimer.limitations.is_empty());
        assert_eq!(disclaimer.warning_level, DisclaimerLevel::Warning);
    }

    #[test]
    fn test_generate_general_disclaimer() {
        let disclaimer = GenericDisclaimerGenerator::generate_general_ai_disclaimer();

        assert!(disclaimer.message.contains("artificial intelligence"));
        assert!(disclaimer.limitations.len() >= 5);
        assert_eq!(disclaimer.warning_level, DisclaimerLevel::Info);
    }

    #[test]
    fn test_generate_high_risk_disclaimer() {
        let disclaimer = GenericDisclaimerGenerator::generate_high_risk_disclaimer();

        assert_eq!(disclaimer.warning_level, DisclaimerLevel::Critical);
        assert!(disclaimer.title.contains("HIGH-RISK"));
    }

    #[test]
    fn test_format_for_display() {
        let disclaimer = GenericDisclaimerGenerator::generate_general_ai_disclaimer();
        let display = GenericDisclaimerGenerator::format_for_display(&disclaimer);

        assert!(display.contains("Limitations:"));
        assert!(display.contains("Recommendations:"));
        assert!(display.contains("═══"));
    }

    #[test]
    fn test_format_inline() {
        let disclaimer = GenericDisclaimerGenerator::generate_unknown_model("test");
        let inline = GenericDisclaimerGenerator::format_inline(&disclaimer);

        assert!(inline.contains("⚠️"));
        assert!(inline.len() < 200); // Should be concise
    }

    #[test]
    fn test_severity_colors() {
        assert_eq!(
            GenericDisclaimerGenerator::get_severity_color(&DisclaimerLevel::Info),
            "#3B82F6"
        );
        assert_eq!(
            GenericDisclaimerGenerator::get_severity_color(&DisclaimerLevel::Warning),
            "#F59E0B"
        );
        assert_eq!(
            GenericDisclaimerGenerator::get_severity_color(&DisclaimerLevel::Critical),
            "#EF4444"
        );
    }

    #[test]
    fn test_ai_act_disclaimer() {
        let disclaimer = GenericDisclaimerGenerator::generate_ai_act_disclaimer();

        assert!(disclaimer.title.contains("EU AI Act"));
        assert!(disclaimer
            .recommendations
            .iter()
            .any(|r| r.contains("right")));
    }
}
