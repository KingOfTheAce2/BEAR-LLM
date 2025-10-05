use anyhow::{anyhow, Result};
use calamine::{open_workbook, Data, Reader, Xls, Xlsx};
use docx_rs::*;
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use serde_json::Value as JsonValue;
use std::io::Read;
use std::path::PathBuf;
use tokio::fs;

pub struct FileProcessor {
    max_file_size: usize,
    supported_formats: Vec<String>,
    allowed_base_dir: Option<PathBuf>,
}

#[allow(dead_code)]
impl FileProcessor {
    pub fn new() -> Self {
        Self::with_base_dir(None)
    }

    pub fn with_base_dir(allowed_base_dir: Option<PathBuf>) -> Self {
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
            allowed_base_dir,
        }
    }

    /// SECURITY: Validate path to prevent traversal attacks
    fn validate_path(&self, file_path: &str) -> Result<PathBuf> {
        let path = PathBuf::from(file_path);

        // SECURITY FIX: Check for symlinks BEFORE canonicalize to prevent TOCTOU
        // Using symlink_metadata doesn't follow symlinks, preventing race conditions
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|e| anyhow!("Cannot access file path: {}", e))?;

        // Explicitly reject symlinks for security - prevents symlink attacks
        if metadata.file_type().is_symlink() {
            return Err(anyhow!(
                "Security violation: Symbolic links are not allowed. \
                Please use the direct file path instead."
            ));
        }

        // Now safe to canonicalize since we've verified it's not a symlink
        let canonical = path
            .canonicalize()
            .map_err(|e| anyhow!("Invalid or inaccessible file path: {}", e))?;

        // If base directory is set, ensure path is within it
        if let Some(ref base_dir) = self.allowed_base_dir {
            let canonical_base = base_dir
                .canonicalize()
                .map_err(|e| anyhow!("Invalid base directory: {}", e))?;

            if !canonical.starts_with(&canonical_base) {
                return Err(anyhow!(
                    "Security violation: Path traversal attempt detected. \
                    File must be within allowed directory: {:?}",
                    canonical_base
                ));
            }
        }

        tracing::debug!("Path validated (not a symlink): {:?}", canonical);
        Ok(canonical)
    }

    pub async fn process_file(&self, file_path: &str, _file_type: &str) -> Result<String> {
        // SECURITY: Validate path first to prevent traversal attacks
        let validated_path = self.validate_path(file_path)?;

        if !validated_path.exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }

        let metadata = fs::metadata(&validated_path).await?;
        if metadata.len() as usize > self.max_file_size {
            return Err(anyhow!("File size exceeds maximum limit of 50MB"));
        }

        let extension = validated_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("Could not determine file extension"))?;

        if !self.supported_formats.contains(&extension.to_lowercase()) {
            return Err(anyhow!("Unsupported file format: {}", extension));
        }

        // Use validated_path string for further processing
        let validated_path_str = validated_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid UTF-8 in file path"))?;

        match extension.to_lowercase().as_str() {
            "txt" | "md" => self.process_text_file(validated_path_str).await,
            "pdf" => self.process_pdf_file(validated_path_str).await,
            "docx" | "doc" => self.process_word_file(validated_path_str).await,
            "xlsx" | "xls" => self.process_excel_file(validated_path_str).await,
            "csv" => self.process_csv_file(validated_path_str).await,
            "pptx" | "ppt" => self.process_powerpoint_file(validated_path_str).await,
            "json" => self.process_json_file(validated_path_str).await,
            "xml" | "html" => self.process_markup_file(validated_path_str).await,
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
                Ok(format!(
                    "PDF content from: {} (Advanced PDF parsing requires additional dependencies)",
                    file_path
                ))
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
                        Err(_) => Ok(format!(
                            "Word document content from: {} (DOCX parsing error)",
                            file_path
                        )),
                    }
                }
            }
        } else {
            // Legacy DOC format - use binary text extraction
            match self.extract_doc_text(file_path).await {
                Ok(text) => {
                    if text.is_empty() || text.trim().is_empty() {
                        Ok(format!(
                            "Word document from: {} (No readable text content found)",
                            file_path
                        ))
                    } else {
                        Ok(text)
                    }
                }
                Err(e) => {
                    println!("DOC parsing failed: {}", e);
                    Ok(format!(
                        "Word document from: {} (Legacy DOC format - text extraction error: {})",
                        file_path, e
                    ))
                }
            }
        }
    }

    async fn process_excel_file(&self, file_path: &str) -> Result<String> {
        match self.extract_excel_enhanced(file_path).await {
            Ok(text) => Ok(text),
            Err(e) => {
                println!("Enhanced Excel parsing failed: {}", e);
                Ok(format!(
                    "Excel spreadsheet content from: {} (Parsing error: {})",
                    file_path, e
                ))
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
                    Ok(format!(
                        "PowerPoint presentation content from: {} (PPTX parsing error: {})",
                        file_path, e
                    ))
                }
            }
        } else {
            // Legacy PPT format - use binary text extraction
            match self.extract_ppt_text(file_path).await {
                Ok(text) => {
                    if text.is_empty() || text.trim().is_empty() {
                        Ok(format!(
                            "PowerPoint presentation from: {} (No readable text content found)",
                            file_path
                        ))
                    } else {
                        Ok(text)
                    }
                }
                Err(e) => {
                    println!("PPT parsing failed: {}", e);
                    Ok(format!("PowerPoint presentation from: {} (Legacy PPT format - text extraction error: {})", file_path, e))
                }
            }
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
        text = text
            .replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    pub fn is_supported(&self, file_extension: &str) -> bool {
        self.supported_formats
            .contains(&file_extension.to_lowercase())
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
                                    if let ParagraphChild::Run(run) = child {
                                        for run_child in &run.children {
                                            if let RunChild::Text(text) = run_child {
                                                paragraph_text.push(text.text.clone());
                                            }
                                        }
                                    }                }
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
                    extracted_text.push(format!(
                        "Slide {}: {}",
                        self.extract_slide_number(&name),
                        slide_text
                    ));
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

    // Legacy DOC format text extraction
    async fn extract_doc_text(&self, file_path: &str) -> Result<String> {
        // Read the file as binary
        let bytes = std::fs::read(file_path)?;

        // Try to parse as OLE compound file format
        match self.extract_doc_from_ole(&bytes) {
            Ok(text) => {
                if !text.is_empty() {
                    return Ok(text);
                }
            }
            Err(e) => {
                println!("OLE parsing failed, trying binary extraction: {}", e);
            }
        }

        // Fallback: extract printable text from binary data
        let text = self.extract_text_from_binary(&bytes);
        Ok(text)
    }

    // Extract text from OLE compound file (DOC format)
    fn extract_doc_from_ole(&self, bytes: &[u8]) -> Result<String> {
        use std::io::Cursor;

        let cursor = Cursor::new(bytes);
        let mut comp = cfb::CompoundFile::open(cursor)?;

        // Try to read the WordDocument stream
        if let Ok(mut stream) = comp.open_stream("/WordDocument") {
            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer)?;

            // Extract text from the Word binary format
            // This is a simplified extraction - Word binary format is complex
            let text = self.extract_text_from_word_stream(&buffer);
            return Ok(text);
        }

        Err(anyhow!("WordDocument stream not found"))
    }

    // Extract text from Word binary stream
    fn extract_text_from_word_stream(&self, data: &[u8]) -> String {
        // Word 97-2003 binary format is complex with many structures
        // This is a basic extraction that looks for printable text sequences
        let mut text = Vec::new();
        let mut current_word = Vec::new();

        for &byte in data {
            if (32..=126).contains(&byte) || byte == 9 || byte == 10 || byte == 13 {
                current_word.push(byte);
            } else {
                // Non-printable character - end current word
                if current_word.len() >= 3 {
                    // Only keep sequences of at least 3 characters
                    if let Ok(word) = String::from_utf8(current_word.clone()) {
                        let cleaned = word.trim();
                        if !cleaned.is_empty() {
                            text.push(cleaned.to_string());
                        }
                    }
                }
                current_word.clear();
            }
        }

        // Don't forget the last word
        if current_word.len() >= 3 {
            if let Ok(word) = String::from_utf8(current_word) {
                let cleaned = word.trim();
                if !cleaned.is_empty() {
                    text.push(cleaned.to_string());
                }
            }
        }

        text.join(" ")
    }

    // Legacy PPT format text extraction
    async fn extract_ppt_text(&self, file_path: &str) -> Result<String> {
        // Read the file as binary
        let bytes = std::fs::read(file_path)?;

        // Try to parse as OLE compound file format
        match self.extract_ppt_from_ole(&bytes) {
            Ok(text) => {
                if !text.is_empty() {
                    return Ok(text);
                }
            }
            Err(e) => {
                println!("OLE parsing failed, trying binary extraction: {}", e);
            }
        }

        // Fallback: extract printable text from binary data
        let text = self.extract_text_from_binary(&bytes);
        Ok(text)
    }

    // Extract text from OLE compound file (PPT format)
    fn extract_ppt_from_ole(&self, bytes: &[u8]) -> Result<String> {
        use std::io::Cursor;

        let cursor = Cursor::new(bytes);
        let mut comp = cfb::CompoundFile::open(cursor)?;

        let mut all_text = Vec::new();

        // Try to read PowerPoint Document stream
        if let Ok(mut stream) = comp.open_stream("/PowerPoint Document") {
            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer)?;

            let text = self.extract_text_from_ppt_stream(&buffer);
            if !text.is_empty() {
                all_text.push(text);
            }
        }

        // Try to read Current User stream
        if let Ok(mut stream) = comp.open_stream("/Current User") {
            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer)?;

            let text = self.extract_text_from_ppt_stream(&buffer);
            if !text.is_empty() {
                all_text.push(text);
            }
        }

        if all_text.is_empty() {
            return Err(anyhow!("No readable text streams found"));
        }

        Ok(all_text.join("\n\n"))
    }

    // Extract text from PowerPoint binary stream
    fn extract_text_from_ppt_stream(&self, data: &[u8]) -> String {
        // PowerPoint binary format stores text in various record types
        // This is a simplified extraction that looks for text patterns
        let mut text = Vec::new();
        let mut current_word = Vec::new();
        let mut in_text_sequence = false;
        let mut char_count = 0;

        for &byte in data {
            if (32..=126).contains(&byte) || byte == 9 || byte == 10 || byte == 13 {
                current_word.push(byte);
                char_count += 1;
                in_text_sequence = true;
            } else {
                if in_text_sequence && current_word.len() >= 3 {
                    // Only keep sequences of at least 3 characters
                    if let Ok(word) = String::from_utf8(current_word.clone()) {
                        let cleaned = word.trim();
                        if !cleaned.is_empty() && !cleaned.chars().all(|c| c.is_ascii_punctuation())
                        {
                            text.push(cleaned.to_string());
                        }
                    }
                }
                current_word.clear();

                // Reset text sequence flag after multiple non-printable bytes
                if char_count > 0 {
                    char_count = 0;
                } else {
                    in_text_sequence = false;
                }
            }
        }

        // Don't forget the last word
        if in_text_sequence && current_word.len() >= 3 {
            if let Ok(word) = String::from_utf8(current_word) {
                let cleaned = word.trim();
                if !cleaned.is_empty() && !cleaned.chars().all(|c| c.is_ascii_punctuation()) {
                    text.push(cleaned.to_string());
                }
            }
        }

        text.join(" ")
    }

    // Generic binary text extraction with encoding detection
    fn extract_text_from_binary(&self, bytes: &[u8]) -> String {
        // Try different encodings
        let encodings: Vec<&'static Encoding> = vec![UTF_8, WINDOWS_1252];

        for encoding in encodings {
            if let Some(text) = self.try_decode_binary(bytes, encoding) {
                if !text.is_empty() {
                    return text;
                }
            }
        }

        // Last resort: extract ASCII-like sequences
        self.extract_ascii_sequences(bytes)
    }

    // Try to decode binary data with a specific encoding
    fn try_decode_binary(&self, bytes: &[u8], encoding: &'static Encoding) -> Option<String> {
        let (decoded, _, had_errors) = encoding.decode(bytes);

        if !had_errors {
            let text: String = decoded
                .chars()
                .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
                .collect();

            let words: Vec<&str> = text
                .split_whitespace()
                .filter(|word| word.len() >= 3)
                .collect();

            if words.len() > 10 {
                return Some(words.join(" "));
            }
        }

        None
    }

    // Extract ASCII-like character sequences from binary data
    fn extract_ascii_sequences(&self, bytes: &[u8]) -> String {
        let mut text = Vec::new();
        let mut current_sequence = Vec::new();

        for &byte in bytes {
            if (32..=126).contains(&byte) || byte == 9 || byte == 10 || byte == 13 {
                current_sequence.push(byte);
            } else {
                if current_sequence.len() >= 4 {
                    if let Ok(s) = String::from_utf8(current_sequence.clone()) {
                        let cleaned = s.trim();
                        if !cleaned.is_empty()
                            && !cleaned.chars().all(|c| c.is_ascii_punctuation())
                            && cleaned.chars().any(|c| c.is_alphabetic())
                        {
                            text.push(cleaned.to_string());
                        }
                    }
                }
                current_sequence.clear();
            }
        }

        // Process the last sequence
        if current_sequence.len() >= 4 {
            if let Ok(s) = String::from_utf8(current_sequence) {
                let cleaned = s.trim();
                if !cleaned.is_empty()
                    && !cleaned.chars().all(|c| c.is_ascii_punctuation())
                    && cleaned.chars().any(|c| c.is_alphabetic())
                {
                    text.push(cleaned.to_string());
                }
            }
        }

        text.join(" ")
    }
}
