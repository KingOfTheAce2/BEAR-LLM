use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use serde_json::Value as JsonValue;

pub struct FileProcessor {
    max_file_size: usize,
    supported_formats: Vec<String>,
}

impl FileProcessor {
    pub fn new() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024, // 50MB
            supported_formats: vec![
                "txt".to_string(),
                "pdf".to_string(),
                "docx".to_string(),
                "doc".to_string(),
                "xlsx".to_string(),
                "xls".to_string(),
                "csv".to_string(),
                "pptx".to_string(),
                "ppt".to_string(),
                "rtf".to_string(),
                "md".to_string(),
                "json".to_string(),
                "xml".to_string(),
                "html".to_string(),
            ],
        }
    }

    pub async fn process_file(&self, file_path: &str, _file_type: &str) -> Result<String> {
        let path = Path::new(file_path);

        if !path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }

        let metadata = fs::metadata(path).await?;
        if metadata.len() as usize > self.max_file_size {
            return Err(anyhow!("File size exceeds maximum limit of 50MB"));
        }

        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("Could not determine file extension"))?;

        if !self.supported_formats.contains(&extension.to_lowercase()) {
            return Err(anyhow!("Unsupported file format: {}", extension));
        }

        match extension.to_lowercase().as_str() {
            "txt" | "md" => self.process_text_file(file_path).await,
            "pdf" => self.process_pdf_file(file_path).await,
            "docx" | "doc" => self.process_word_file(file_path).await,
            "xlsx" | "xls" => self.process_excel_file(file_path).await,
            "csv" => self.process_csv_file(file_path).await,
            "pptx" | "ppt" => self.process_powerpoint_file(file_path).await,
            "json" => self.process_json_file(file_path).await,
            "xml" | "html" => self.process_markup_file(file_path).await,
            _ => Err(anyhow!("Unsupported file type: {}", extension)),
        }
    }

    async fn process_text_file(&self, file_path: &str) -> Result<String> {
        let content = fs::read_to_string(file_path).await?;
        Ok(content)
    }

    async fn process_pdf_file(&self, file_path: &str) -> Result<String> {
        Ok(format!("PDF content from: {} (PDF parsing to be implemented)", file_path))
    }

    async fn process_word_file(&self, file_path: &str) -> Result<String> {
        Ok(format!("Word document content from: {} (DOCX parsing to be implemented)", file_path))
    }

    async fn process_excel_file(&self, file_path: &str) -> Result<String> {
        Ok(format!("Excel spreadsheet content from: {} (Excel parsing to be implemented)", file_path))
    }

    async fn process_csv_file(&self, file_path: &str) -> Result<String> {
        let content = fs::read_to_string(file_path).await?;
        Ok(content)
    }

    async fn process_powerpoint_file(&self, file_path: &str) -> Result<String> {
        Ok(format!("PowerPoint presentation content from: {} (PPTX parsing to be implemented)", file_path))
    }

    async fn process_json_file(&self, file_path: &str) -> Result<String> {
        let content = fs::read_to_string(file_path).await?;
        let json: JsonValue = serde_json::from_str(&content)?;
        Ok(serde_json::to_string_pretty(&json)?)
    }

    async fn process_markup_file(&self, file_path: &str) -> Result<String> {
        let content = fs::read_to_string(file_path).await?;
        let text = self.strip_html_tags(&content);
        Ok(text)
    }

    fn strip_html_tags(&self, html: &str) -> String {
        let tag_regex = regex::Regex::new(r"<[^>]+>").unwrap();
        let script_regex = regex::Regex::new(r"(?s)<script[^>]*>.*?</script>").unwrap();
        let style_regex = regex::Regex::new(r"(?s)<style[^>]*>.*?</style>").unwrap();

        let mut text = script_regex.replace_all(html, "").to_string();
        text = style_regex.replace_all(&text, "").to_string();
        text = tag_regex.replace_all(&text, " ").to_string();
        text = text.replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    pub fn is_supported(&self, file_extension: &str) -> bool {
        self.supported_formats.contains(&file_extension.to_lowercase())
    }

    pub fn get_supported_formats(&self) -> Vec<String> {
        self.supported_formats.clone()
    }
}