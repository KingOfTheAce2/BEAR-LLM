use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCard {
    pub model_id: String,
    pub description: String,
    pub intended_use: Vec<String>,
    pub limitations: Vec<String>,
    pub biases: Vec<String>,
    pub training_data: Option<String>,
    pub license: Option<String>,
    pub paper_url: Option<String>,
    pub ethical_considerations: Vec<String>,
    pub safety_warnings: Vec<String>,
    pub performance_metrics: Vec<String>,
}

pub struct ModelCardParser;

impl ModelCardParser {
    /// Parse model card from markdown content
    pub fn parse(model_id: String, markdown: &str) -> ModelCard {
        let sections = Self::extract_sections(markdown);

        ModelCard {
            model_id: model_id.clone(),
            description: Self::extract_description(&sections, markdown),
            intended_use: Self::extract_list_items(
                &sections,
                &["intended use", "uses", "use cases"],
            ),
            limitations: Self::extract_list_items(
                &sections,
                &["limitations", "known issues", "caveats"],
            ),
            biases: Self::extract_list_items(&sections, &["bias", "biases", "fairness"]),
            training_data: Self::extract_section_content(
                &sections,
                &["training data", "dataset", "data"],
            ),
            license: Self::extract_license(&sections, markdown),
            paper_url: Self::extract_paper_url(markdown),
            ethical_considerations: Self::extract_list_items(
                &sections,
                &["ethical", "ethics", "responsible use"],
            ),
            safety_warnings: Self::extract_safety_warnings(markdown),
            performance_metrics: Self::extract_metrics(markdown),
        }
    }

    /// Extract sections from markdown
    fn extract_sections(markdown: &str) -> Vec<(String, String)> {
        let mut sections = Vec::new();
        let header_re = Regex::new(r"(?m)^#{1,6}\s+(.+)$").unwrap();

        let mut current_section = ("".to_string(), String::new());
        let mut last_pos = 0;

        for cap in header_re.captures_iter(markdown) {
            if let Some(matched) = cap.get(0) {
                // Save previous section
                if !current_section.0.is_empty() {
                    let content = markdown[last_pos..matched.start()].trim().to_string();
                    current_section.1 = content;
                    sections.push(current_section.clone());
                }

                // Start new section
                let title = cap[1].trim().to_lowercase();
                current_section = (title, String::new());
                last_pos = matched.end();
            }
        }

        // Save last section
        if !current_section.0.is_empty() {
            let content = markdown[last_pos..].trim().to_string();
            current_section.1 = content;
            sections.push(current_section);
        }

        sections
    }

    /// Extract description (usually first paragraph or model description section)
    fn extract_description(sections: &[(String, String)], markdown: &str) -> String {
        // Try to find a description section
        for (title, content) in sections {
            if title.contains("description") || title.contains("overview") {
                return Self::extract_first_paragraph(content);
            }
        }

        // Fall back to first paragraph
        Self::extract_first_paragraph(markdown)
    }

    /// Extract first non-empty paragraph
    fn extract_first_paragraph(text: &str) -> String {
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with('|')
                && !trimmed.starts_with('-')
                && trimmed.len() > 20
            {
                return trimmed.to_string();
            }
        }
        "No description available.".to_string()
    }

    /// Extract content from a specific section
    fn extract_section_content(sections: &[(String, String)], keywords: &[&str]) -> Option<String> {
        for (title, content) in sections {
            for keyword in keywords {
                if title.contains(keyword) {
                    return Some(content.trim().to_string());
                }
            }
        }
        None
    }

    /// Extract list items from sections
    fn extract_list_items(sections: &[(String, String)], keywords: &[&str]) -> Vec<String> {
        let mut items = Vec::new();

        // Compile regex patterns once outside the loop
        let bullet_re = Regex::new(r"(?m)^[\s-]*[-*•]\s+(.+)$").unwrap();
        let num_re = Regex::new(r"(?m)^\s*\d+\.\s+(.+)$").unwrap();

        for (title, content) in sections {
            for keyword in keywords {
                if title.contains(keyword) {
                    // Extract bullet points
                    for cap in bullet_re.captures_iter(content) {
                        if let Some(item) = cap.get(1) {
                            items.push(item.as_str().trim().to_string());
                        }
                    }

                    // If no bullet points, try numbered lists
                    if items.is_empty() {
                        for cap in num_re.captures_iter(content) {
                            if let Some(item) = cap.get(1) {
                                items.push(item.as_str().trim().to_string());
                            }
                        }
                    }

                    // If still no items, split by newlines (for simple lists)
                    if items.is_empty() {
                        for line in content.lines() {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() && trimmed.len() > 10 {
                                items.push(trimmed.to_string());
                            }
                        }
                    }
                }
            }
        }

        items
    }

    /// Extract license information
    fn extract_license(sections: &[(String, String)], markdown: &str) -> Option<String> {
        // Try sections first
        if let Some(content) = Self::extract_section_content(sections, &["license"]) {
            return Some(content);
        }

        // Try to find license in metadata
        let license_re = Regex::new(r"(?i)license:\s*([^\n]+)").unwrap();
        if let Some(cap) = license_re.captures(markdown) {
            if let Some(license) = cap.get(1) {
                return Some(license.as_str().trim().to_string());
            }
        }

        None
    }

    /// Extract paper URL
    fn extract_paper_url(markdown: &str) -> Option<String> {
        let paper_re = Regex::new(r"(?i)paper:\s*(https?://[^\s\)]+)").unwrap();
        if let Some(cap) = paper_re.captures(markdown) {
            if let Some(url) = cap.get(1) {
                return Some(url.as_str().trim().to_string());
            }
        }

        // Try to find arxiv links
        let arxiv_re = Regex::new(r"(https?://arxiv\.org/abs/[^\s\)]+)").unwrap();
        if let Some(cap) = arxiv_re.captures(markdown) {
            if let Some(url) = cap.get(1) {
                return Some(url.as_str().trim().to_string());
            }
        }

        None
    }

    /// Extract safety warnings
    fn extract_safety_warnings(markdown: &str) -> Vec<String> {
        let mut warnings = Vec::new();

        // Look for warning patterns
        let warning_patterns = [
            r"(?i)⚠️\s*([^\n]+)",
            r"(?i)warning:\s*([^\n]+)",
            r"(?i)caution:\s*([^\n]+)",
            r"(?i)note:\s*([^\n]+)",
        ];

        for pattern in &warning_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for cap in re.captures_iter(markdown) {
                    if let Some(warning) = cap.get(1) {
                        warnings.push(warning.as_str().trim().to_string());
                    }
                }
            }
        }

        warnings
    }

    /// Extract performance metrics
    fn extract_metrics(markdown: &str) -> Vec<String> {
        let mut metrics = Vec::new();

        // Look for tables with metrics
        let table_row_re = Regex::new(r"(?m)^\|[^\|]+\|[^\|]+\|").unwrap();
        for cap in table_row_re.captures_iter(markdown) {
            if let Some(row) = cap.get(0) {
                let row_str = row.as_str().trim();
                if !row_str.contains("---") && !row_str.to_lowercase().contains("metric") {
                    metrics.push(row_str.to_string());
                }
            }
        }

        // Look for percentage/accuracy mentions
        let metric_re =
            Regex::new(r"(?i)(accuracy|f1|bleu|rouge|perplexity)[:\s]+([0-9.]+%?)").unwrap();
        for cap in metric_re.captures_iter(markdown) {
            if let (Some(name), Some(value)) = (cap.get(1), cap.get(2)) {
                metrics.push(format!("{}: {}", name.as_str(), value.as_str()));
            }
        }

        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_model_card() {
        let markdown = r#"
# Llama 2 7B Chat

Llama 2 is a collection of pretrained and fine-tuned generative text models.

## Intended Use

- Conversational AI applications
- Research purposes
- Educational demonstrations

## Limitations

- May produce inaccurate information (hallucinations)
- Limited knowledge cutoff date
- Not suitable for critical decision making

## Bias

- Training data may contain societal biases
- May exhibit gender and cultural stereotypes

## Training Data

Trained on 2 trillion tokens from publicly available sources.

## License

License: Llama 2 Community License

⚠️ Warning: This model may generate harmful content.
"#;

        let card = ModelCardParser::parse("meta-llama/Llama-2-7b-chat-hf".to_string(), markdown);

        assert_eq!(card.model_id, "meta-llama/Llama-2-7b-chat-hf");
        assert!(card.description.contains("Llama 2"));
        assert_eq!(card.intended_use.len(), 3);
        assert_eq!(card.limitations.len(), 3);
        assert_eq!(card.biases.len(), 2);
        assert!(card.training_data.is_some());
        assert!(card.license.is_some());
        assert_eq!(card.safety_warnings.len(), 1);
    }

    #[test]
    fn test_extract_sections() {
        let markdown = r#"
# Title

Content 1

## Section 1

Content 2

### Subsection

Content 3
"#;

        let sections = ModelCardParser::extract_sections(markdown);
        assert!(sections.len() >= 2);
    }

    #[test]
    fn test_extract_license() {
        let markdown = "License: MIT\n\nSome other content.";
        let sections = vec![];

        let license = ModelCardParser::extract_license(&sections, markdown);
        assert_eq!(license, Some("MIT".to_string()));
    }

    #[test]
    fn test_extract_paper_url() {
        let markdown = "Paper: https://arxiv.org/abs/2307.09288";

        let url = ModelCardParser::extract_paper_url(markdown);
        assert_eq!(url, Some("https://arxiv.org/abs/2307.09288".to_string()));
    }
}
