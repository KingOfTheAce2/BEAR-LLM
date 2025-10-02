/// AI Transparency Notice Templates
///
/// Provides standardized notices and disclaimers for AI Act compliance.
use serde::{Deserialize, Serialize};

/// Collection of transparency notices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeTemplates {
    pub startup: StartupNotice,
    pub onboarding: OnboardingNotice,
    pub limitations: LimitationsNotice,
    pub data_processing: DataProcessingNotice,
    pub legal_disclaimer: LegalDisclaimers,
}

impl Default for NoticeTemplates {
    fn default() -> Self {
        Self {
            startup: StartupNotice::default(),
            onboarding: OnboardingNotice::default(),
            limitations: LimitationsNotice::default(),
            data_processing: DataProcessingNotice::default(),
            legal_disclaimer: LegalDisclaimers::default(),
        }
    }
}

/// Startup disclaimer shown on application launch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupNotice {
    pub title: String,
    pub content: Vec<String>,
    pub acknowledgment_required: bool,
}

impl Default for StartupNotice {
    fn default() -> Self {
        Self {
            title: "AI-Powered Legal Assistant".to_string(),
            content: vec![
                "🤖 This application uses Artificial Intelligence to assist with legal research and document analysis.".to_string(),
                "".to_string(),
                "Important Information:".to_string(),
                "• All content is AI-generated and may contain errors or inaccuracies".to_string(),
                "• AI outputs should always be reviewed by qualified legal professionals".to_string(),
                "• This tool does NOT provide legal advice and should not be used as a substitute for professional counsel".to_string(),
                "• Your data is processed locally for privacy, in compliance with GDPR and EU AI Act".to_string(),
                "".to_string(),
                "By using this application, you acknowledge these limitations and agree to use AI-generated content responsibly.".to_string(),
            ],
            acknowledgment_required: true,
        }
    }
}

impl StartupNotice {
    pub fn to_formatted_string(&self) -> String {
        let mut output = format!("{}\n{}\n\n", self.title, "=".repeat(self.title.len()));
        output.push_str(&self.content.join("\n"));
        output
    }
}

/// First-time user onboarding notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingNotice {
    pub title: String,
    pub sections: Vec<OnboardingSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingSection {
    pub heading: String,
    pub content: String,
    pub icon: String,
}

impl Default for OnboardingNotice {
    fn default() -> Self {
        Self {
            title: "Welcome to BEAR AI - Legal Assistant".to_string(),
            sections: vec![
                OnboardingSection {
                    heading: "What is BEAR AI?".to_string(),
                    icon: "🐻".to_string(),
                    content: "BEAR AI is a privacy-focused AI assistant designed specifically for legal professionals. It helps with document analysis, research, and drafting tasks while keeping your data secure and private.".to_string(),
                },
                OnboardingSection {
                    heading: "AI Capabilities".to_string(),
                    icon: "🤖".to_string(),
                    content: "The AI can analyze documents, answer questions, and assist with drafting. However, it has limitations:\n• May produce inaccurate or incomplete information\n• Cannot replace professional legal judgment\n• Requires human oversight for all legal work\n• Performance varies based on complexity and context".to_string(),
                },
                OnboardingSection {
                    heading: "Privacy & Security".to_string(),
                    icon: "🔒".to_string(),
                    content: "Your data privacy is our priority:\n• All processing happens locally on your device\n• No data is sent to external servers\n• GDPR and EU AI Act compliant\n• PII detection and protection enabled\n• Full data portability and deletion rights".to_string(),
                },
                OnboardingSection {
                    heading: "Responsible Use".to_string(),
                    icon: "⚖️".to_string(),
                    content: "Always use AI-generated content responsibly:\n• Review all AI outputs before use\n• Never rely solely on AI for legal decisions\n• Verify facts and citations independently\n• Maintain professional oversight\n• Be aware of AI limitations and biases".to_string(),
                },
                OnboardingSection {
                    heading: "EU AI Act Compliance".to_string(),
                    icon: "🇪🇺".to_string(),
                    content: "This application complies with the EU AI Act:\n• High-risk AI system safeguards in place\n• Transparency obligations fulfilled\n• Human oversight mechanisms enabled\n• Quality management and documentation\n• Regular risk assessments conducted".to_string(),
                },
            ],
        }
    }
}

impl OnboardingNotice {
    pub fn to_formatted_string(&self) -> String {
        let mut output = format!("{}\n{}\n\n", self.title, "=".repeat(self.title.len()));

        for section in &self.sections {
            output.push_str(&format!("{} {}\n", section.icon, section.heading));
            output.push_str(&format!("{}\n\n", section.content));
        }

        output
    }
}

/// Model limitations disclosure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitationsNotice {
    pub title: String,
    pub limitations: Vec<Limitation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limitation {
    pub category: String,
    pub description: String,
    pub severity: LimitationSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LimitationSeverity {
    Critical,
    Important,
    Moderate,
    Minor,
}

impl Default for LimitationsNotice {
    fn default() -> Self {
        Self {
            title: "AI Model Limitations & Known Issues".to_string(),
            limitations: vec![
                Limitation {
                    category: "Factual Accuracy".to_string(),
                    description: "AI may generate plausible-sounding but incorrect information. Always verify facts, case law, and legal precedents independently.".to_string(),
                    severity: LimitationSeverity::Critical,
                },
                Limitation {
                    category: "Legal Advice".to_string(),
                    description: "This AI does NOT provide legal advice. Outputs are informational only and must be reviewed by qualified legal professionals.".to_string(),
                    severity: LimitationSeverity::Critical,
                },
                Limitation {
                    category: "Bias & Fairness".to_string(),
                    description: "AI models may reflect biases present in training data. Be mindful of potential discrimination or unfair treatment in outputs.".to_string(),
                    severity: LimitationSeverity::Important,
                },
                Limitation {
                    category: "Context Understanding".to_string(),
                    description: "AI may misunderstand nuanced legal contexts, jurisdictional differences, or specialized terminology. Review outputs carefully.".to_string(),
                    severity: LimitationSeverity::Important,
                },
                Limitation {
                    category: "Currency of Information".to_string(),
                    description: "AI training data has a cutoff date. Recent legal developments, new legislation, or case law may not be reflected.".to_string(),
                    severity: LimitationSeverity::Important,
                },
                Limitation {
                    category: "Complex Reasoning".to_string(),
                    description: "AI may struggle with multi-step legal reasoning, edge cases, or novel legal questions requiring creative analysis.".to_string(),
                    severity: LimitationSeverity::Moderate,
                },
                Limitation {
                    category: "Citation Accuracy".to_string(),
                    description: "AI-generated citations may be incorrect, outdated, or fabricated. Always verify all citations independently.".to_string(),
                    severity: LimitationSeverity::Critical,
                },
            ],
        }
    }
}

impl LimitationsNotice {
    pub fn to_formatted_string(&self) -> String {
        let mut output = format!("{}\n{}\n\n", self.title, "=".repeat(self.title.len()));

        for limitation in &self.limitations {
            let severity_icon = match limitation.severity {
                LimitationSeverity::Critical => "❌",
                LimitationSeverity::Important => "⚠️",
                LimitationSeverity::Moderate => "ℹ️",
                LimitationSeverity::Minor => "💡",
            };

            output.push_str(&format!("{} {}\n", severity_icon, limitation.category));
            output.push_str(&format!("   {}\n\n", limitation.description));
        }

        output
    }
}

/// Data processing transparency notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessingNotice {
    pub title: String,
    pub sections: Vec<DataProcessingSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessingSection {
    pub heading: String,
    pub content: String,
}

impl Default for DataProcessingNotice {
    fn default() -> Self {
        Self {
            title: "Data Processing & Privacy".to_string(),
            sections: vec![
                DataProcessingSection {
                    heading: "Local Processing".to_string(),
                    content: "All AI processing occurs locally on your device. No data is transmitted to external servers or third parties.".to_string(),
                },
                DataProcessingSection {
                    heading: "Data Storage".to_string(),
                    content: "Conversations and documents are stored locally in an encrypted database. You maintain full control over your data.".to_string(),
                },
                DataProcessingSection {
                    heading: "PII Detection".to_string(),
                    content: "Automatic detection and protection of Personally Identifiable Information (PII) including names, addresses, ID numbers, and sensitive data.".to_string(),
                },
                DataProcessingSection {
                    heading: "GDPR Rights".to_string(),
                    content: "You have the right to access, rectify, delete, and port your data. Export and deletion features are available in settings.".to_string(),
                },
                DataProcessingSection {
                    heading: "Data Retention".to_string(),
                    content: "Data is retained locally until you choose to delete it. No automatic data transmission or cloud synchronization occurs.".to_string(),
                },
                DataProcessingSection {
                    heading: "Model Training".to_string(),
                    content: "Your data is NOT used to train AI models. All processing uses pre-trained models that run locally.".to_string(),
                },
            ],
        }
    }
}

impl DataProcessingNotice {
    pub fn to_formatted_string(&self) -> String {
        let mut output = format!("{}\n{}\n\n", self.title, "=".repeat(self.title.len()));

        for section in &self.sections {
            output.push_str(&format!("📋 {}\n", section.heading));
            output.push_str(&format!("   {}\n\n", section.content));
        }

        output
    }
}

/// Legal context disclaimers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalDisclaimers {
    pub general: String,
    pub high_risk: String,
    pub citation: String,
    pub professional_review: String,
}

impl Default for LegalDisclaimers {
    fn default() -> Self {
        Self {
            general: "⚠️ DISCLAIMER: This is AI-generated content for informational purposes only. It does not constitute legal advice and should not be relied upon for legal decisions. Always consult qualified legal professionals for legal matters.".to_string(),

            high_risk: "❌ HIGH RISK: This content relates to legal advice or decisions that may affect legal rights. MANDATORY professional review required before use. Do not act on this information without consulting a qualified attorney.".to_string(),

            citation: "⚠️ CITATION NOTICE: AI-generated citations may be incorrect, outdated, or fabricated. ALWAYS verify all citations, case law, and legal references independently through authoritative sources.".to_string(),

            professional_review: "⚖️ PROFESSIONAL OVERSIGHT REQUIRED: As a high-risk AI system under the EU AI Act, all outputs from this application must be reviewed by qualified legal professionals before use in legal contexts.".to_string(),
        }
    }
}

impl LegalDisclaimers {
    /// Get appropriate disclaimer based on context
    pub fn get_disclaimer(&self, is_high_risk: bool, includes_citations: bool) -> String {
        let mut disclaimer = self.general.clone();

        if is_high_risk {
            disclaimer.push_str(&format!("\n\n{}", self.high_risk));
        }

        if includes_citations {
            disclaimer.push_str(&format!("\n\n{}", self.citation));
        }

        if is_high_risk {
            disclaimer.push_str(&format!("\n\n{}", self.professional_review));
        }

        disclaimer
    }
}

#[cfg(test)]
mod notice_tests {
    use super::*;

    #[test]
    fn test_startup_notice_formatting() {
        let notice = StartupNotice::default();
        let formatted = notice.to_formatted_string();
        assert!(formatted.contains("AI-Powered Legal Assistant"));
        assert!(formatted.contains("does NOT provide legal advice"));
    }

    #[test]
    fn test_limitations_severity() {
        let notice = LimitationsNotice::default();
        let critical_count = notice
            .limitations
            .iter()
            .filter(|l| l.severity == LimitationSeverity::Critical)
            .count();
        assert!(critical_count > 0);
    }

    #[test]
    fn test_legal_disclaimer_context() {
        let disclaimers = LegalDisclaimers::default();

        let low_risk = disclaimers.get_disclaimer(false, false);
        assert!(!low_risk.contains("HIGH RISK"));

        let high_risk = disclaimers.get_disclaimer(true, true);
        assert!(high_risk.contains("HIGH RISK"));
        assert!(high_risk.contains("CITATION NOTICE"));
    }
}
