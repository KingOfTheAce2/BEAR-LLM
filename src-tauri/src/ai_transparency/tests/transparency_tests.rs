/// Comprehensive Tests for AI Transparency Module

#[cfg(test)]
mod transparency_module_tests {
    use crate::ai_transparency::*;
    use crate::ai_transparency::confidence::*;
    use crate::ai_transparency::notices::*;

    #[test]
    fn test_risk_level_classification() {
        // Test all risk level combinations
        assert_eq!(
            RiskLevel::from_context(true, true),
            RiskLevel::High,
            "Legal advice affecting rights should be high risk"
        );

        assert_eq!(
            RiskLevel::from_context(true, false),
            RiskLevel::Limited,
            "Legal advice not affecting rights should be limited risk"
        );

        assert_eq!(
            RiskLevel::from_context(false, true),
            RiskLevel::Limited,
            "Non-legal affecting rights should be limited risk"
        );

        assert_eq!(
            RiskLevel::from_context(false, false),
            RiskLevel::Minimal,
            "Neither legal nor affecting rights should be minimal risk"
        );
    }

    #[test]
    fn test_risk_level_oversight_requirements() {
        assert!(RiskLevel::High.requires_human_oversight());
        assert!(RiskLevel::Unacceptable.requires_human_oversight());
        assert!(!RiskLevel::Limited.requires_human_oversight());
        assert!(!RiskLevel::Minimal.requires_human_oversight());
    }

    #[test]
    fn test_risk_level_warning_messages() {
        let high_warning = RiskLevel::High.warning_message();
        assert!(high_warning.contains("professional review"));
        assert!(high_warning.contains("legal contexts"));

        let minimal_warning = RiskLevel::Minimal.warning_message();
        assert!(minimal_warning.contains("informational"));
    }

    #[test]
    fn test_transparency_context_creation() {
        let context = TransparencyContext::new("llama-3-70b", RiskLevel::High);

        assert_eq!(context.model_name, "llama-3-70b");
        assert_eq!(context.risk_level, RiskLevel::High);
        assert!(context.requires_human_oversight);
        assert!(context.ai_generated);
        assert!(!context.disclaimers_acknowledged);
        assert!(!context.interaction_id.is_empty());
    }

    #[test]
    fn test_transparency_context_confidence() {
        let context = TransparencyContext::new("test-model", RiskLevel::Minimal)
            .with_confidence(0.85);

        assert_eq!(context.confidence, 0.85);

        // Test clamping
        let clamped_high = TransparencyContext::new("test", RiskLevel::Minimal)
            .with_confidence(1.5);
        assert_eq!(clamped_high.confidence, 1.0);

        let clamped_low = TransparencyContext::new("test", RiskLevel::Minimal)
            .with_confidence(-0.5);
        assert_eq!(clamped_low.confidence, 0.0);
    }

    #[test]
    fn test_transparency_context_acknowledgment() {
        let context = TransparencyContext::new("test", RiskLevel::High)
            .acknowledge_disclaimers();

        assert!(context.disclaimers_acknowledged);
    }

    #[test]
    fn test_transparency_notice_generation() {
        let context = TransparencyContext::new("test-model", RiskLevel::High)
            .with_confidence(0.75);

        let notice = context.get_notice();

        assert!(notice.contains("AI-Generated"));
        assert!(notice.contains("test-model"));
        assert!(notice.contains("75%"));
        assert!(notice.contains("Professional review required"));
    }

    #[test]
    fn test_transparency_preferences_default() {
        let prefs = TransparencyPreferences::default();

        assert!(prefs.show_startup_disclaimer);
        assert!(prefs.show_response_notices);
        assert!(prefs.show_confidence);
        assert!(!prefs.onboarding_completed);
        assert!(prefs.last_acknowledgment.is_none());
    }

    #[test]
    fn test_transparency_preferences_needs_disclaimer() {
        let prefs = TransparencyPreferences::default();
        assert!(prefs.needs_disclaimer(), "New user should need disclaimer");

        let completed = prefs.complete_onboarding();
        assert!(!completed.needs_disclaimer(), "After onboarding should not need disclaimer");
        assert!(completed.onboarding_completed);
        assert!(completed.last_acknowledgment.is_some());
    }

    #[test]
    fn test_confidence_level_categorization() {
        assert_eq!(ConfidenceLevel::from_score(0.95), ConfidenceLevel::VeryHigh);
        assert_eq!(ConfidenceLevel::from_score(0.9), ConfidenceLevel::VeryHigh);
        assert_eq!(ConfidenceLevel::from_score(0.85), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.75), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.65), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.5), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.35), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.25), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.15), ConfidenceLevel::VeryLow);
    }

    #[test]
    fn test_confidence_level_indicators() {
        assert_eq!(ConfidenceLevel::VeryHigh.indicator(), "🟢 Very High");
        assert_eq!(ConfidenceLevel::High.indicator(), "🟢 High");
        assert_eq!(ConfidenceLevel::Medium.indicator(), "🟡 Medium");
        assert_eq!(ConfidenceLevel::Low.indicator(), "🟠 Low");
        assert_eq!(ConfidenceLevel::VeryLow.indicator(), "🔴 Very Low");
    }

    #[test]
    fn test_confidence_level_recommendations() {
        let rec = ConfidenceLevel::VeryLow.recommendation();
        assert!(rec.contains("extreme caution"));

        let rec_high = ConfidenceLevel::VeryHigh.recommendation();
        assert!(rec_high.contains("verify"));
    }

    #[test]
    fn test_confidence_factors_calculation() {
        let factors = ConfidenceFactors {
            completeness: 0.8,
            context_understanding: 0.7,
            factual_consistency: 0.9,
            coherence: 0.8,
            source_reliability: 0.7,
        };

        let overall = factors.calculate_overall();
        assert!(overall > 0.7 && overall < 0.85);
    }

    #[test]
    fn test_confidence_factors_from_metadata() {
        let factors = ConfidenceFactors::from_response_metadata(
            200,    // Long response
            true,   // Has citations
            1500,   // Large context
            Some(0.85),
        );

        assert!(factors.completeness >= 0.8);
        assert!(factors.context_understanding >= 0.8);
        assert!(factors.source_reliability > 0.4);
        assert_eq!(factors.coherence, 0.85);
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

        assert!(score.overall > 0.0 && score.overall <= 1.0);
        assert!(!score.explanation.is_empty());
    }

    #[test]
    fn test_confidence_score_display() {
        let factors = ConfidenceFactors::default_medium();
        let score = ConfidenceScore::new(factors);

        let display = score.format_display();
        assert!(display.contains("Confidence level"));
        assert!(display.contains("Factors:"));
        assert!(display.contains("Completeness"));
    }

    #[test]
    fn test_startup_notice_content() {
        let notice = StartupNotice::default();

        assert!(notice.acknowledgment_required);
        assert!(notice.title.contains("AI"));

        let formatted = notice.to_formatted_string();
        assert!(formatted.contains("does NOT provide legal advice"));
        assert!(formatted.contains("GDPR"));
        assert!(formatted.contains("EU AI Act"));
    }

    #[test]
    fn test_onboarding_notice_sections() {
        let notice = OnboardingNotice::default();

        assert!(!notice.sections.is_empty());
        assert!(notice.sections.iter().any(|s| s.heading.contains("Privacy")));
        assert!(notice.sections.iter().any(|s| s.heading.contains("EU AI Act")));

        let formatted = notice.to_formatted_string();
        assert!(formatted.contains("🐻"));
        assert!(formatted.contains("🔒"));
    }

    #[test]
    fn test_limitations_notice_severity() {
        let notice = LimitationsNotice::default();

        let critical = notice.limitations.iter()
            .filter(|l| l.severity == LimitationSeverity::Critical)
            .count();

        assert!(critical >= 2, "Should have multiple critical limitations");

        let formatted = notice.to_formatted_string();
        assert!(formatted.contains("❌")); // Critical indicator
        assert!(formatted.contains("Citation"));
    }

    #[test]
    fn test_data_processing_notice_gdpr() {
        let notice = DataProcessingNotice::default();

        assert!(notice.sections.iter().any(|s| s.heading.contains("GDPR")));
        assert!(notice.sections.iter().any(|s| s.heading.contains("PII")));

        let formatted = notice.to_formatted_string();
        assert!(formatted.contains("local"));
        assert!(formatted.contains("No data is transmitted"));
    }

    #[test]
    fn test_legal_disclaimers_context() {
        let disclaimers = LegalDisclaimers::default();

        // Low risk scenario
        let low = disclaimers.get_disclaimer(false, false);
        assert!(low.contains("informational purposes"));
        assert!(!low.contains("HIGH RISK"));

        // High risk with citations
        let high = disclaimers.get_disclaimer(true, true);
        assert!(high.contains("HIGH RISK"));
        assert!(high.contains("CITATION NOTICE"));
        assert!(high.contains("PROFESSIONAL OVERSIGHT"));

        // High risk without citations
        let high_no_cite = disclaimers.get_disclaimer(true, false);
        assert!(high_no_cite.contains("HIGH RISK"));
        assert!(!high_no_cite.contains("CITATION NOTICE"));
    }

    #[test]
    fn test_notice_templates_default() {
        let templates = NoticeTemplates::default();

        assert!(!templates.startup.content.is_empty());
        assert!(!templates.onboarding.sections.is_empty());
        assert!(!templates.limitations.limitations.is_empty());
        assert!(!templates.data_processing.sections.is_empty());
    }

    #[test]
    fn test_factual_consistency_weighting() {
        // High factual consistency should strongly influence score
        let high_factual = ConfidenceFactors {
            completeness: 0.5,
            context_understanding: 0.5,
            factual_consistency: 0.95,
            coherence: 0.5,
            source_reliability: 0.5,
        };

        // Low factual consistency should strongly lower score
        let low_factual = ConfidenceFactors {
            completeness: 0.9,
            context_understanding: 0.9,
            factual_consistency: 0.3,
            coherence: 0.9,
            source_reliability: 0.9,
        };

        let high_score = high_factual.calculate_overall();
        let low_score = low_factual.calculate_overall();

        // Factual consistency is weighted heavily for legal context
        assert!(high_score > 0.5);
        assert!(low_score < 0.75);
    }
}
