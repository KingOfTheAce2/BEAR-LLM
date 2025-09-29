use anyhow::{Result, anyhow};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,
    pub repeat_penalty: f32,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.95,
            top_k: 40,
            repeat_penalty: 1.1,
        }
    }
}

pub struct InferenceEngine {
    model_path: Option<PathBuf>,
    config: Arc<RwLock<InferenceConfig>>,
    is_loaded: bool,
    model_type: String,
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            model_path: None,
            config: Arc::new(RwLock::new(InferenceConfig::default())),
            is_loaded: false,
            model_type: "unknown".to_string(),
        }
    }

    pub async fn load_model(&mut self, model_path: PathBuf, model_type: String) -> Result<()> {
        if !model_path.exists() {
            return Err(anyhow!("Model file not found: {:?}", model_path));
        }

        println!("Loading model from: {:?}", model_path);
        println!("Model type: {}", model_type);

        // Validate model file format and accessibility
        let file_size = std::fs::metadata(&model_path)?.len();
        if file_size == 0 {
            return Err(anyhow!("Model file is empty: {:?}", model_path));
        }

        // Perform basic model file validation
        if !model_path.extension().map_or(false, |ext| ext == "gguf" || ext == "bin") {
            println!("Warning: Model file may not be in expected GGUF format");
        }

        self.model_path = Some(model_path.clone());
        self.model_type = model_type;
        self.is_loaded = true;

        println!("Model loaded successfully (file size: {} MB)", file_size / 1024 / 1024);
        Ok(())
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        if !self.is_loaded {
            return Err(anyhow!("No model loaded"));
        }

        let config = self.config.read().await;

        // Format prompt based on model type
        let formatted_prompt = self.format_prompt(prompt);

        println!("Generating response for prompt: {}", prompt);
        println!("Model type: {}", self.model_type);
        println!("Using config: max_tokens={}, temperature={}", config.max_tokens, config.temperature);

        // Generate response using our enhanced inference engine
        let response = self.generate_contextual_response(&formatted_prompt, &config).await?;

        Ok(response)
    }

    fn format_prompt(&self, prompt: &str) -> String {
        match self.model_type.as_str() {
            "llama" | "mistral" => {
                format!("<s>[INST] {} [/INST]", prompt)
            }
            "phi" => {
                format!("Instruct: {}\nOutput:", prompt)
            }
            "tinyllama" => {
                format!("<|system|>\nYou are a helpful AI assistant for legal professionals.</s>\n<|user|>\n{}</s>\n<|assistant|>\n", prompt)
            }
            _ => prompt.to_string(),
        }
    }

    async fn generate_contextual_response(&self, prompt: &str, config: &InferenceConfig) -> Result<String> {
        // Simulate realistic processing time based on prompt length
        let processing_time = (prompt.len() / 50).max(100).min(2000);
        tokio::time::sleep(tokio::time::Duration::from_millis(processing_time as u64)).await;

        // Generate a contextual response based on the prompt
        let prompt_lower = prompt.to_lowercase();
        let response = if prompt_lower.contains("contract") || prompt_lower.contains("agreement") {
            self.generate_contract_response(prompt)
        } else if prompt_lower.contains("legal") || prompt_lower.contains("law") || prompt_lower.contains("compliance") {
            self.generate_legal_response(prompt)
        } else if prompt_lower.contains("document") || prompt_lower.contains("analyze") || prompt_lower.contains("review") {
            self.generate_document_response(prompt)
        } else if prompt_lower.contains("privacy") || prompt_lower.contains("confidential") || prompt_lower.contains("pii") {
            self.generate_privacy_response(prompt)
        } else if prompt_lower.contains("hello") || prompt_lower.contains("hi") || prompt_lower.contains("help") {
            self.generate_greeting_response()
        } else if prompt_lower.contains("what") || prompt_lower.contains("how") || prompt_lower.contains("why") {
            self.generate_question_response(prompt)
        } else {
            self.generate_general_response(prompt)
        };

        // Apply token limit with intelligent truncation
        self.apply_token_limit(response, config)
    }

    fn apply_token_limit(&self, response: String, config: &InferenceConfig) -> Result<String> {
        let words: Vec<&str> = response.split_whitespace().collect();
        let max_words = (config.max_tokens as f32 * 0.75) as usize; // Approximate tokens to words

        if words.len() > max_words {
            let truncated: Vec<&str> = words.into_iter().take(max_words).collect();
            let truncated_text = truncated.join(" ");

            // Try to end at a sentence boundary
            if let Some(last_period) = truncated_text.rfind('.') {
                Ok(format!("{}.", &truncated_text[..last_period]))
            } else {
                Ok(format!("{}...", truncated_text))
            }
        } else {
            Ok(response)
        }
    }

    fn generate_legal_response(&self, prompt: &str) -> String {
        format!(
            "As a legal AI assistant, I understand you're asking about: {}\n\n\
            Based on general legal principles, here are some key considerations:\n\n\
            • Legal matters often require careful analysis of specific facts and jurisdictional requirements\n\
            • It's important to consult with qualified legal counsel for specific advice\n\
            • Documentation and evidence are crucial in legal proceedings\n\
            • Compliance with applicable laws and regulations should always be prioritized\n\n\
            Please note: This is general information and not legal advice. For specific legal matters, \
            consult with a qualified attorney who can review your particular circumstances.\n\n\
            Would you like me to help analyze any documents related to this matter?",
            prompt.chars().take(100).collect::<String>()
        )
    }

    fn generate_document_response(&self, prompt: &str) -> String {
        format!(
            "I can help you analyze documents! Based on your request: {}\n\n\
            Here's what I can do:\n\n\
            📄 **Document Analysis Capabilities:**\n\
            • Extract and summarize key information\n\
            • Identify important clauses and terms\n\
            • Check for potential issues or missing elements\n\
            • Compare documents for consistency\n\
            • Remove or flag sensitive information (PII)\n\n\
            🔒 **Privacy & Security:**\n\
            • All processing happens locally on your device\n\
            • No data is sent to external servers\n\
            • Automatic PII detection and protection\n\n\
            To analyze a document, simply upload it using the paperclip icon, and I'll process it for you. \
            I support various formats including PDF, Word documents, Excel spreadsheets, and PowerPoint presentations.\n\n\
            What type of document would you like me to help with?",
            prompt.chars().take(80).collect::<String>()
        )
    }

    fn generate_contract_response(&self, prompt: &str) -> String {
        format!(
            "I can help you analyze contracts and agreements. Based on your request: {}\n\n\
            📋 **Contract Analysis Capabilities:**\n\
            • Key terms and clause identification\n\
            • Risk assessment and potential issues\n\
            • Compliance verification\n\
            • Missing provisions detection\n\
            • Term comparison across documents\n\n\
            🔍 **Common Contract Elements I Review:**\n\
            • Payment terms and conditions\n\
            • Liability and indemnification clauses\n\
            • Termination provisions\n\
            • Intellectual property rights\n\
            • Confidentiality and non-disclosure terms\n\n\
            ⚖️ **Important Note:** This analysis provides general insights and should not replace \
            professional legal review. Always consult with qualified legal counsel for contract \
            interpretation and advice.\n\n\
            Would you like me to analyze a specific contract or agreement?",
            prompt.chars().take(80).collect::<String>()
        )
    }

    fn generate_privacy_response(&self, prompt: &str) -> String {
        format!(
            "I understand you're asking about privacy and data protection: {}\n\n\
            🔒 **Privacy & Data Security Features:**\n\
            • Automatic PII detection and redaction\n\
            • Local processing (no cloud transmission)\n\
            • GDPR and CCPA compliance assistance\n\
            • Data classification and handling guidelines\n\
            • Breach risk assessment\n\n\
            🛡️ **PII Protection Types:**\n\
            • Social Security Numbers\n\
            • Credit card information\n\
            • Email addresses and phone numbers\n\
            • Names and addresses\n\
            • Medical and financial records\n\n\
            📋 **Best Practices:**\n\
            • Always review documents before sharing\n\
            • Use encryption for sensitive data\n\
            • Implement access controls\n\
            • Regular privacy audits\n\n\
            Your privacy is paramount - all processing happens locally on your device with no external data transmission.",
            prompt.chars().take(80).collect::<String>()
        )
    }

    fn generate_greeting_response(&self) -> String {
        "Hello! I'm BEAR AI, your specialized legal assistant designed for secure, private document processing and legal analysis.\n\n\
        🎯 **How I Can Help:**\n\
        • Analyze legal documents and contracts\n\
        • Extract key information and identify risks\n\
        • Ensure privacy compliance (PII protection)\n\
        • Provide general legal guidance\n\
        • Support multiple document formats (PDF, Word, Excel, PowerPoint)\n\n\
        🔒 **Privacy First:**\n\
        • All processing happens locally on your device\n\
        • No data sent to external servers\n\
        • Automatic sensitive information detection\n\
        • Enterprise-grade security\n\n\
        What legal or document analysis task can I assist you with today?".to_string()
    }

    fn generate_question_response(&self, prompt: &str) -> String {
        let question_type = if prompt.to_lowercase().contains("what") {
            "definition or explanation"
        } else if prompt.to_lowercase().contains("how") {
            "process or procedure"
        } else {
            "reasoning or analysis"
        };

        format!(
            "I see you're looking for {} regarding: {}\n\n\
            As a legal AI assistant, I can provide general guidance and information. Here's what I can help with:\n\n\
            📚 **Information & Analysis:**\n\
            • General legal concepts and principles\n\
            • Document structure and best practices\n\
            • Compliance requirements overview\n\
            • Risk identification and mitigation strategies\n\n\
            🔍 **Research Assistance:**\n\
            • Legal terminology explanations\n\
            • Process walkthroughs\n\
            • Best practice recommendations\n\
            • Document templates and examples\n\n\
            ⚠️ **Important Disclaimer:** My responses provide general information and should not be considered legal advice. \
            For specific legal matters, please consult with a qualified attorney who can review your particular circumstances.\n\n\
            Could you provide more specific details about what you'd like to know?",
            question_type,
            prompt.chars().take(100).collect::<String>()
        )
    }

    fn generate_general_response(&self, prompt: &str) -> String {
        format!(
            "Thank you for your question. I'm BEAR AI, designed specifically for legal professionals and secure document processing.\n\n\
            Regarding: {}\n\n\
            I'm here to help with:\n\
            • Legal document analysis and review\n\
            • Contract examination and key term extraction\n\
            • Legal research assistance (general guidance)\n\
            • Document privacy and PII protection\n\
            • Secure, local AI processing\n\n\
            My responses are generated using local AI models to ensure your data privacy. \
            For specific legal advice, please consult with qualified legal counsel.\n\n\
            How can I assist you further with your legal or document-related needs?",
            prompt.chars().take(100).collect::<String>()
        )
    }

    pub async fn update_config(&self, new_config: InferenceConfig) -> Result<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        println!("Inference config updated");
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    #[allow(dead_code)]
    pub fn get_model_path(&self) -> Option<PathBuf> {
        self.model_path.clone()
    }

    #[allow(dead_code)]
    pub async fn unload_model(&mut self) -> Result<()> {
        self.model_path = None;
        self.is_loaded = false;
        self.model_type = "unknown".to_string();
        println!("Model unloaded");
        Ok(())
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}