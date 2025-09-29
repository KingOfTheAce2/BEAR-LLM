use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use serde_json::Value as JsonValue;
use calamine::{Reader, Xlsx, Xls, open_workbook, Data};
use docx_rs::*;

pub struct FileProcessor {
    max_file_size: usize,
    supported_formats: Vec<String>,
}

#[allow(dead_code)]
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
        // PDF parsing using pdf-extract crate
        match pdf_extract::extract_text(file_path) {
            Ok(text) => Ok(text),
            Err(e) => {
                // Fallback to basic text extraction
                println!("PDF parsing failed, using fallback: {}", e);
                Ok(format!("PDF content from: {} (Advanced PDF parsing requires additional dependencies)", file_path))
            }
        }
    }

    async fn process_word_file(&self, file_path: &str) -> Result<String> {
        if file_path.ends_with(".docx") {
            match self.extract_docx_enhanced(file_path).await {
                Ok(text) => Ok(text),
                Err(e) => {
                    println!("Enhanced DOCX parsing failed, using fallback: {}", e);
                    // Fallback to basic ZIP extraction
                    match self.extract_docx_text(file_path).await {
                        Ok(text) => Ok(text),
                        Err(_) => Ok(format!("Word document content from: {} (DOCX parsing error)", file_path))
                    }
                }
            }
        } else {
            // DOC files require more complex parsing - use basic extraction for now
            Ok(format!("Word document content from: {} (Legacy DOC format support limited)", file_path))
        }
    }

    async fn process_excel_file(&self, file_path: &str) -> Result<String> {
        match self.extract_excel_enhanced(file_path).await {
            Ok(text) => Ok(text),
            Err(e) => {
                println!("Enhanced Excel parsing failed: {}", e);
                Ok(format!("Excel spreadsheet content from: {} (Parsing error: {})", file_path, e))
            }
        }
    }

    async fn process_csv_file(&self, file_path: &str) -> Result<String> {
        let content = fs::read_to_string(file_path).await?;
        Ok(content)
    }

    async fn process_powerpoint_file(&self, file_path: &str) -> Result<String> {
        if file_path.ends_with(".pptx") {
            match self.extract_pptx_text(file_path).await {
                Ok(text) => Ok(text),
                Err(e) => {
                    println!("PPTX parsing failed: {}", e);
                    Ok(format!("PowerPoint presentation content from: {} (PPTX parsing error: {})", file_path, e))
                }
            }
        } else {
            Ok(format!("PowerPoint presentation content from: {} (Legacy PPT format support limited)", file_path))
        }
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

    // Helper method for DOCX text extraction
    async fn extract_docx_text(&self, file_path: &str) -> Result<String> {
        use std::io::Read;

        // Basic DOCX text extraction by reading document.xml from the ZIP
        let file = std::fs::File::open(file_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut document_xml = archive.by_name("word/document.xml")?;
        let mut xml_content = String::new();
        document_xml.read_to_string(&mut xml_content)?;

        // Extract text between <w:t> tags (very basic parsing)
        let text = self.extract_text_from_xml(&xml_content);
        Ok(text)
    }

    // Enhanced Excel text extraction using calamine crate
    async fn extract_excel_enhanced(&self, file_path: &str) -> Result<String> {
        let path = std::path::Path::new(file_path);
        let mut extracted_text = Vec::new();

        if file_path.ends_with(".xlsx") {
            let mut workbook: Xlsx<_> = open_workbook(path)?;
            let sheet_names = workbook.sheet_names().to_vec();

            for sheet_name in sheet_names {
                if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                    extracted_text.push(format!("Sheet: {}", sheet_name));
                    extracted_text.push(self.range_to_text(&range));
                }
            }
        } else if file_path.ends_with(".xls") {
            let mut workbook: Xls<_> = open_workbook(path)?;
            let sheet_names = workbook.sheet_names().to_vec();

            for sheet_name in sheet_names {
                if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                    extracted_text.push(format!("Sheet: {}", sheet_name));
                    extracted_text.push(self.range_to_text(&range));
                }
            }
        }

        Ok(extracted_text.join("\n\n"))
    }

    // Helper method to convert Excel range to text
    fn range_to_text(&self, range: &calamine::Range<Data>) -> String {
        let mut rows = Vec::new();

        for row in range.rows() {
            let mut row_text = Vec::new();
            for cell in row {
                match cell {
                    Data::Empty => row_text.push("".to_string()),
                    Data::String(s) => row_text.push(s.clone()),
                    Data::Float(f) => row_text.push(f.to_string()),
                    Data::Int(i) => row_text.push(i.to_string()),
                    Data::Bool(b) => row_text.push(b.to_string()),
                    Data::Error(e) => row_text.push(format!("ERROR: {:?}", e)),
                    Data::DateTime(dt) => row_text.push(dt.to_string()),
                    Data::DateTimeIso(dt) => row_text.push(dt.to_string()),
                    Data::DurationIso(d) => row_text.push(d.to_string()),
                }
            }
            if !row_text.iter().all(|s| s.is_empty()) {
                rows.push(row_text.join("\t"));
            }
        }

        rows.join("\n")
    }

    // Enhanced DOCX text extraction using docx-rs crate
    async fn extract_docx_enhanced(&self, file_path: &str) -> Result<String> {
        let content = std::fs::read(file_path)?;
        let docx = read_docx(&content)?;

        let mut extracted_text = Vec::new();

        // Extract text from document body
        for child in &docx.document.children {
            extracted_text.push(self.extract_text_from_element(child));
        }

        Ok(extracted_text.join("\n"))
    }

    // Helper method to extract text from docx elements
    fn extract_text_from_element(&self, element: &DocumentChild) -> String {
        match element {
            DocumentChild::Paragraph(paragraph) => {
                let mut paragraph_text = Vec::new();
                for child in &paragraph.children {
                    match child {
                        ParagraphChild::Run(run) => {
                            for run_child in &run.children {
                                if let RunChild::Text(text) = run_child {
                                    paragraph_text.push(text.text.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                paragraph_text.join("")
            }
            DocumentChild::Table(_table) => {
                // Simplified table handling - extract basic table indicator
                "[TABLE_CONTENT]".to_string()
            }
            _ => String::new(),
        }
    }

    // PPTX text extraction using ZIP-based approach
    async fn extract_pptx_text(&self, file_path: &str) -> Result<String> {
        use std::io::Read;

        let file = std::fs::File::open(file_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut extracted_text = Vec::new();

        // Look for slide files in the ZIP archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            // Process slide XML files
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                let mut xml_content = String::new();
                file.read_to_string(&mut xml_content)?;

                // Extract text from slide XML
                let slide_text = self.extract_pptx_slide_text(&xml_content);
                if !slide_text.is_empty() {
                    extracted_text.push(format!("Slide {}: {}",
                        self.extract_slide_number(&name), slide_text));
                }
            }
        }

        Ok(extracted_text.join("\n\n"))
    }

    // Helper method to extract text from PPTX slide XML
    fn extract_pptx_slide_text(&self, xml: &str) -> String {
        use regex::Regex;

        // Extract text between <a:t> and </a:t> tags (text runs in PowerPoint)
        let text_regex = Regex::new(r"<a:t[^>]*>([^<]*)</a:t>").unwrap();
        let mut extracted_text = Vec::new();

        for cap in text_regex.captures_iter(xml) {
            if let Some(text) = cap.get(1) {
                let text_content = text.as_str().trim();
                if !text_content.is_empty() {
                    extracted_text.push(text_content);
                }
            }
        }

        extracted_text.join(" ")
    }

    // Helper method to extract slide number from filename
    fn extract_slide_number(&self, filename: &str) -> String {
        use regex::Regex;

        let slide_regex = Regex::new(r"slide(\d+)\.xml").unwrap();
        if let Some(cap) = slide_regex.captures(filename) {
            if let Some(num) = cap.get(1) {
                return num.as_str().to_string();
            }
        }
        "Unknown".to_string()
    }

    // Helper method to extract text from XML
    fn extract_text_from_xml(&self, xml: &str) -> String {
        use regex::Regex;

        // Extract text between <w:t> and </w:t> tags
        let text_regex = Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
        let mut extracted_text = Vec::new();

        for cap in text_regex.captures_iter(xml) {
            if let Some(text) = cap.get(1) {
                extracted_text.push(text.as_str());
            }
        }

        extracted_text.join(" ")
    }
}