use bear_ai_llm::ai_transparency::{
    ModelCardParser, DisclaimerGenerator, GenericDisclaimerGenerator,
    DisclaimerLevel,
};

#[test]
fn test_parse_complete_model_card() {
    let markdown = r#"
# Llama 2 7B Chat

Llama 2 is a collection of pretrained and fine-tuned generative text models ranging in scale from 7 billion to 70 billion parameters.

## Model Description

This is the 7B parameter chat-optimized variant of Llama 2.

## Intended Use

The model is intended for:
- Commercial and research use in English
- Assistant-like chat applications
- Natural language processing research

## Limitations and Biases

- May produce inaccurate information (hallucinations)
- Limited knowledge cutoff date (September 2022)
- Not suitable for critical decision making
- May exhibit biases from training data

## Training Data

Trained on 2 trillion tokens of data from publicly available sources.

## License

License: Llama 2 Community License Agreement

## Citation

Paper: https://arxiv.org/abs/2307.09288

⚠️ Warning: This model may generate harmful or biased content.
"#;

    let card = ModelCardParser::parse("meta-llama/Llama-2-7b-chat-hf".to_string(), markdown);

    assert_eq!(card.model_id, "meta-llama/Llama-2-7b-chat-hf");
    assert!(card.description.contains("Llama 2"));
    assert_eq!(card.intended_use.len(), 3);
    assert!(card.intended_use[0].contains("Commercial"));
    assert_eq!(card.limitations.len(), 4);
    assert!(card.limitations[0].contains("inaccurate"));
    assert!(card.training_data.is_some());
    assert!(card.license.is_some());
    assert_eq!(card.safety_warnings.len(), 1);
    assert!(card.paper_url.is_some());
}

#[test]
fn test_parse_minimal_model_card() {
    let markdown = r#"
# Test Model

A simple test model.
"#;

    let card = ModelCardParser::parse("test/model".to_string(), markdown);

    assert_eq!(card.model_id, "test/model");
    assert!(card.description.contains("simple test model"));
    // Should have default/empty values
    assert!(card.intended_use.is_empty());
    assert!(card.limitations.is_empty());
}

#[test]
fn test_extract_license_from_metadata() {
    let markdown = r#"
---
license: MIT
---

# Model
"#;

    let card = ModelCardParser::parse("test/model".to_string(), markdown);
    assert!(card.license.is_some());
}

#[test]
fn test_extract_arxiv_paper() {
    let markdown = r#"
Read more in our paper: https://arxiv.org/abs/2307.09288
"#;

    let card = ModelCardParser::parse("test/model".to_string(), markdown);
    assert_eq!(
        card.paper_url,
        Some("https://arxiv.org/abs/2307.09288".to_string())
    );
}

#[test]
fn test_disclaimer_generator() {
    let markdown = r#"
# Test Model

## Intended Use
- Research
- Education

## Limitations
- May hallucinate
- Limited knowledge

⚠️ Use with caution
"#;

    let card = ModelCardParser::parse("test/model".to_string(), markdown);
    let disclaimer = DisclaimerGenerator::generate(&card);

    assert_eq!(disclaimer.model_id, "test/model");
    assert!(!disclaimer.warnings.is_empty());
    assert_eq!(disclaimer.capabilities.len(), 2);
    assert!(!disclaimer.limitations.is_empty());
}

#[test]
fn test_disclaimer_acknowledgment_required() {
    let markdown = r#"
# Risky Model

## Limitations
- Major limitation 1
- Major limitation 2
- Major limitation 3
- Major limitation 4

## Bias
- Significant bias issue

⚠️ Warning 1
⚠️ Warning 2
"#;

    let card = ModelCardParser::parse("risky/model".to_string(), markdown);
    let disclaimer = DisclaimerGenerator::generate(&card);

    assert!(disclaimer.acknowledgment_required);
}

#[test]
fn test_generic_disclaimer_unknown_model() {
    let disclaimer = GenericDisclaimerGenerator::generate_unknown_model("unknown-model");

    assert!(disclaimer.title.contains("Unavailable"));
    assert_eq!(disclaimer.warning_level, DisclaimerLevel::Warning);
    assert!(disclaimer.limitations.len() >= 5);
    assert!(!disclaimer.recommendations.is_empty());
}

#[test]
fn test_generic_disclaimer_offline() {
    let disclaimer = GenericDisclaimerGenerator::generate_offline_disclaimer("test-model");

    assert!(disclaimer.title.contains("Offline"));
    assert_eq!(disclaimer.warning_level, DisclaimerLevel::Warning);
    assert!(disclaimer.message.contains("test-model"));
}

#[test]
fn test_high_risk_disclaimer() {
    let disclaimer = GenericDisclaimerGenerator::generate_high_risk_disclaimer();

    assert_eq!(disclaimer.warning_level, DisclaimerLevel::Critical);
    assert!(disclaimer.title.contains("HIGH-RISK"));
    assert!(disclaimer.recommendations.iter().any(|r| r.contains("NEVER")));
}

#[test]
fn test_ai_act_disclaimer() {
    let disclaimer = GenericDisclaimerGenerator::generate_ai_act_disclaimer();

    assert!(disclaimer.title.contains("EU AI Act"));
    assert_eq!(disclaimer.warning_level, DisclaimerLevel::Info);
    assert!(disclaimer.recommendations.iter().any(|r| r.contains("right")));
}

#[test]
fn test_format_disclaimer_display() {
    let disclaimer = GenericDisclaimerGenerator::generate_general_ai_disclaimer();
    let display = GenericDisclaimerGenerator::format_for_display(&disclaimer);

    assert!(display.contains("═══"));
    assert!(display.contains("Limitations:"));
    assert!(display.contains("Recommendations:"));
}

#[test]
fn test_format_inline_disclaimer() {
    let disclaimer = GenericDisclaimerGenerator::generate_unknown_model("test");
    let inline = GenericDisclaimerGenerator::format_inline(&disclaimer);

    assert!(inline.len() < 200);
    assert!(inline.contains("⚠️") || inline.contains("🚨") || inline.contains("ℹ️"));
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
