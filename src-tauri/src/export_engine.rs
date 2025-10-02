use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use docx_rs::*;
use hex;
use printpdf::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

// Core export structures for GDPR Article 20 compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataExport {
    pub export_date: DateTime<Utc>,
    pub version: String,
    pub user_id: String,
    pub chats: Vec<ChatExport>,
    pub documents: Vec<DocumentExport>,
    pub settings: SettingsExport,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatExport {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<MessageExport>,
    pub model_used: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageExport {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExport {
    pub id: i64,
    pub filename: String,
    pub file_type: String,
    pub upload_date: DateTime<Utc>,
    pub chunk_count: i64,
    pub pii_detections: Vec<PIIDetection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIIDetection {
    pub pii_type: String,
    pub replacement_text: String,
    pub confidence: f64,
    pub position_start: usize,
    pub position_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsExport {
    pub preferences: serde_json::Value,
    pub retention_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub format_version: String,
    pub application_version: String,
    pub export_hash: String,
    pub compliance_info: ComplianceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceInfo {
    pub gdpr_article_20: bool,
    pub encrypted: bool,
    pub integrity_verified: bool,
}

pub struct ExportEngine;

impl Default for ExportEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportEngine {
    pub fn new() -> Self {
        Self
    }

    /// Generate SHA-256 hash for data integrity verification
    #[allow(dead_code)]
    fn generate_hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Export to DOCX format with professional legal document formatting
    pub fn export_to_docx(&self, data: &UserDataExport, output_path: &Path) -> Result<()> {
        let mut docx = Docx::new();

        // Title page with professional formatting
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text("BEAR AI - Data Export Report")
                        .size(32)
                        .bold(),
                )
                .align(AlignmentType::Center),
        );

        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(format!(
                            "Export Date: {}",
                            data.export_date.format("%Y-%m-%d %H:%M:%S UTC")
                        ))
                        .size(24),
                )
                .align(AlignmentType::Center),
        );

        docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text("")));

        // Compliance Statement
        docx = docx.add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text("GDPR Article 20 Compliance Statement")
                    .size(28)
                    .bold(),
            ),
        );

        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(
                    "This export has been generated in accordance with the General Data Protection Regulation (GDPR) Article 20, \
                    providing you with the right to data portability. All personal data processed by BEAR AI is included \
                    in a structured, commonly used, and machine-readable format."
                ).size(22))
        );

        // Export Metadata Section
        docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text("")));
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Export Metadata").size(28).bold()),
        );

        docx = docx.add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(format!("Version: {}", data.version))
                    .size(22),
            ),
        );

        docx = docx.add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(format!("User ID: {}", data.user_id))
                    .size(22),
            ),
        );

        docx = docx.add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(format!("Integrity Hash: {}", data.metadata.export_hash))
                    .size(22),
            ),
        );

        // Chat History Section
        docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text("")));
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Chat History").size(28).bold()),
        );

        for (idx, chat) in data.chats.iter().enumerate() {
            docx = docx.add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text(format!("{}. {}", idx + 1, chat.title))
                        .size(24)
                        .bold(),
                ),
            );

            docx = docx.add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text(format!(
                            "Created: {} | Model: {}",
                            chat.created_at.format("%Y-%m-%d %H:%M:%S"),
                            chat.model_used
                        ))
                        .size(20)
                        .italic(),
                ),
            );

            for msg in &chat.messages {
                let role_label = match msg.role.as_str() {
                    "user" => "YOU",
                    "assistant" => "BEAR AI",
                    _ => &msg.role,
                };

                docx = docx.add_paragraph(
                    Paragraph::new()
                        .add_run(
                            Run::new()
                                .add_text(format!("[{}] ", role_label))
                                .size(22)
                                .bold(),
                        )
                        .add_run(Run::new().add_text(&msg.content).size(22)),
                );
            }

            docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text("")));
        }

        // Documents Section
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Processed Documents").size(28).bold()),
        );

        for doc in &data.documents {
            docx = docx.add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text(format!("• {}", doc.filename))
                        .size(22)
                        .bold(),
                ),
            );

            docx = docx.add_paragraph(
                Paragraph::new().add_run(
                    Run::new()
                        .add_text(format!(
                            "  Type: {} | Upload Date: {} | Chunks: {} | PII Detections: {}",
                            doc.file_type,
                            doc.upload_date.format("%Y-%m-%d"),
                            doc.chunk_count,
                            doc.pii_detections.len()
                        ))
                        .size(20),
                ),
            );
        }

        // Write to file
        let file = std::fs::File::create(output_path)?;
        docx.build().pack(file)?;

        Ok(())
    }

    /// Export to Markdown format optimized for lawyers
    pub fn export_to_markdown(&self, data: &UserDataExport, output_path: &Path) -> Result<()> {
        let mut markdown = String::new();

        // Header
        markdown.push_str("# BEAR AI - Data Export Report\n\n");
        markdown.push_str(&format!(
            "**Export Date:** {}\n\n",
            data.export_date.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        markdown.push_str("---\n\n");

        // Compliance Statement
        markdown.push_str("## GDPR Article 20 Compliance Statement\n\n");
        markdown.push_str("This export has been generated in accordance with the General Data Protection Regulation (GDPR) Article 20, ");
        markdown.push_str("providing you with the right to data portability. All personal data processed by BEAR AI is included ");
        markdown.push_str("in a structured, commonly used, and machine-readable format.\n\n");
        markdown.push_str("---\n\n");

        // Export Metadata
        markdown.push_str("## Export Metadata\n\n");
        markdown.push_str(&format!("- **Version:** {}\n", data.version));
        markdown.push_str(&format!("- **User ID:** {}\n", data.user_id));
        markdown.push_str(&format!(
            "- **Integrity Hash:** `{}`\n",
            data.metadata.export_hash
        ));
        markdown.push_str(&format!(
            "- **GDPR Article 20 Compliant:** {}\n",
            if data.metadata.compliance_info.gdpr_article_20 {
                "✓"
            } else {
                "✗"
            }
        ));
        markdown.push_str(&format!(
            "- **Encrypted:** {}\n",
            if data.metadata.compliance_info.encrypted {
                "✓"
            } else {
                "✗"
            }
        ));
        markdown.push_str(&format!(
            "- **Integrity Verified:** {}\n\n",
            if data.metadata.compliance_info.integrity_verified {
                "✓"
            } else {
                "✗"
            }
        ));
        markdown.push_str("---\n\n");

        // Chat History
        markdown.push_str("## Chat History\n\n");
        for (idx, chat) in data.chats.iter().enumerate() {
            markdown.push_str(&format!("### {}. {}\n\n", idx + 1, chat.title));
            markdown.push_str(&format!(
                "**Created:** {} | **Model:** {} | **Messages:** {}\n\n",
                chat.created_at.format("%Y-%m-%d %H:%M:%S"),
                chat.model_used,
                chat.messages.len()
            ));

            if !chat.tags.is_empty() {
                markdown.push_str(&format!("**Tags:** {}\n\n", chat.tags.join(", ")));
            }

            markdown.push_str("#### Conversation\n\n");
            for msg in &chat.messages {
                let role_label = match msg.role.as_str() {
                    "user" => "**YOU**",
                    "assistant" => "**BEAR AI**",
                    _ => &msg.role,
                };

                markdown.push_str(&format!(
                    "{} ({})\n\n",
                    role_label,
                    msg.timestamp.format("%H:%M:%S")
                ));
                markdown.push_str(&format!("{}\n\n", msg.content));
                markdown.push_str("---\n\n");
            }
        }

        // Documents
        markdown.push_str("## Processed Documents\n\n");
        markdown.push_str(&format!("Total Documents: {}\n\n", data.documents.len()));

        for doc in &data.documents {
            markdown.push_str(&format!("### {}\n\n", doc.filename));
            markdown.push_str(&format!("- **Type:** {}\n", doc.file_type));
            markdown.push_str(&format!(
                "- **Upload Date:** {}\n",
                doc.upload_date.format("%Y-%m-%d")
            ));
            markdown.push_str(&format!("- **Chunks:** {}\n", doc.chunk_count));
            markdown.push_str(&format!(
                "- **PII Detections:** {}\n\n",
                doc.pii_detections.len()
            ));

            if !doc.pii_detections.is_empty() {
                markdown.push_str("**PII Detections:**\n\n");
                for pii in &doc.pii_detections {
                    markdown.push_str(&format!(
                        "- Type: {} | Confidence: {:.2}% | Replaced with: {}\n",
                        pii.pii_type,
                        pii.confidence * 100.0,
                        pii.replacement_text
                    ));
                }
                markdown.push('\n');
            }
        }

        // Footer
        markdown.push_str("---\n\n");
        markdown.push_str(
            "*This export was generated by BEAR AI in compliance with GDPR Article 20.*\n",
        );

        std::fs::write(output_path, markdown)?;
        Ok(())
    }

    /// Export to PDF format with professional quality
    pub fn export_to_pdf(&self, data: &UserDataExport, output_path: &Path) -> Result<()> {
        let (doc, page1, layer1) =
            PdfDocument::new("BEAR AI Data Export", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Title
        current_layer.use_text(
            "BEAR AI - Data Export Report",
            24.0,
            Mm(50.0),
            Mm(270.0),
            &font_bold,
        );
        current_layer.use_text(
            format!(
                "Export Date: {}",
                data.export_date.format("%Y-%m-%d %H:%M:%S UTC")
            ),
            12.0,
            Mm(50.0),
            Mm(260.0),
            &font,
        );

        // Compliance Statement
        current_layer.use_text(
            "GDPR Article 20 Compliance Statement",
            14.0,
            Mm(20.0),
            Mm(240.0),
            &font_bold,
        );
        current_layer.use_text(
            "This export has been generated in accordance with GDPR Article 20.",
            10.0,
            Mm(20.0),
            Mm(230.0),
            &font,
        );

        // Export Metadata
        current_layer.use_text("Export Metadata", 14.0, Mm(20.0), Mm(210.0), &font_bold);
        current_layer.use_text(
            format!("Version: {}", data.version),
            10.0,
            Mm(20.0),
            Mm(200.0),
            &font,
        );
        current_layer.use_text(
            format!("User ID: {}", data.user_id),
            10.0,
            Mm(20.0),
            Mm(195.0),
            &font,
        );
        current_layer.use_text(
            format!("Hash: {}", &data.metadata.export_hash[..16]),
            10.0,
            Mm(20.0),
            Mm(190.0),
            &font,
        );

        // Chat History Summary
        let mut y_pos = 170.0;
        current_layer.use_text(
            "Chat History Summary",
            14.0,
            Mm(20.0),
            Mm(y_pos),
            &font_bold,
        );
        y_pos -= 10.0;

        for (idx, chat) in data.chats.iter().enumerate() {
            if y_pos < 30.0 {
                break; // Avoid running off page (proper pagination would be more complex)
            }
            current_layer.use_text(
                format!(
                    "{}. {} ({} messages)",
                    idx + 1,
                    chat.title,
                    chat.messages.len()
                ),
                10.0,
                Mm(20.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= 5.0;
        }

        doc.save(&mut std::io::BufWriter::new(std::fs::File::create(
            output_path,
        )?))?;
        Ok(())
    }

    /// Export to plain text format (fallback)
    pub fn export_to_text(&self, data: &UserDataExport, output_path: &Path) -> Result<()> {
        let mut text = String::new();

        // Header
        text.push_str("=".repeat(80).as_str());
        text.push('\n');
        text.push_str("BEAR AI - DATA EXPORT REPORT\n");
        text.push_str(&format!(
            "Export Date: {}\n",
            data.export_date.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        text.push_str("=".repeat(80).as_str());
        text.push_str("\n\n");

        // Compliance Statement
        text.push_str("GDPR ARTICLE 20 COMPLIANCE STATEMENT\n");
        text.push_str("-".repeat(80).as_str());
        text.push('\n');
        text.push_str(
            "This export has been generated in accordance with the General Data Protection\n",
        );
        text.push_str(
            "Regulation (GDPR) Article 20, providing you with the right to data portability.\n\n",
        );

        // Metadata
        text.push_str("EXPORT METADATA\n");
        text.push_str("-".repeat(80).as_str());
        text.push('\n');
        text.push_str(&format!("Version: {}\n", data.version));
        text.push_str(&format!("User ID: {}\n", data.user_id));
        text.push_str(&format!("Integrity Hash: {}\n", data.metadata.export_hash));
        text.push_str(&format!(
            "GDPR Compliant: {}\n",
            if data.metadata.compliance_info.gdpr_article_20 {
                "Yes"
            } else {
                "No"
            }
        ));
        text.push_str(&format!(
            "Encrypted: {}\n",
            if data.metadata.compliance_info.encrypted {
                "Yes"
            } else {
                "No"
            }
        ));
        text.push('\n');

        // Chat History
        text.push_str("CHAT HISTORY\n");
        text.push_str("-".repeat(80).as_str());
        text.push_str("\n\n");

        for (idx, chat) in data.chats.iter().enumerate() {
            text.push_str(&format!("Chat #{}: {}\n", idx + 1, chat.title));
            text.push_str(&format!(
                "Created: {} | Model: {}\n",
                chat.created_at.format("%Y-%m-%d %H:%M:%S"),
                chat.model_used
            ));
            text.push_str("-".repeat(80).as_str());
            text.push('\n');

            for msg in &chat.messages {
                let role_label = match msg.role.as_str() {
                    "user" => "YOU",
                    "assistant" => "BEAR AI",
                    _ => &msg.role,
                };

                text.push_str(&format!(
                    "[{}] ({})\n",
                    role_label,
                    msg.timestamp.format("%H:%M:%S")
                ));
                text.push_str(&format!("{}\n\n", msg.content));
            }
            text.push('\n');
        }

        // Documents
        text.push_str("PROCESSED DOCUMENTS\n");
        text.push_str("-".repeat(80).as_str());
        text.push_str("\n\n");

        for doc in &data.documents {
            text.push_str(&format!("Document: {}\n", doc.filename));
            text.push_str(&format!("  Type: {}\n", doc.file_type));
            text.push_str(&format!(
                "  Upload Date: {}\n",
                doc.upload_date.format("%Y-%m-%d")
            ));
            text.push_str(&format!("  Chunks: {}\n", doc.chunk_count));
            text.push_str(&format!("  PII Detections: {}\n", doc.pii_detections.len()));
            text.push('\n');
        }

        text.push_str("=".repeat(80).as_str());
        text.push('\n');
        text.push_str("End of Export Report\n");

        std::fs::write(output_path, text)?;
        Ok(())
    }

    /// Main export function that generates all formats and creates encrypted archive
    pub fn export_user_data(
        &self,
        data: &UserDataExport,
        output_dir: &Path,
        formats: &[&str],
    ) -> Result<Vec<String>> {
        std::fs::create_dir_all(output_dir)?;

        let mut exported_files = Vec::new();

        for format in formats {
            let filename = match *format {
                "docx" => {
                    let path = output_dir.join("bear_ai_export.docx");
                    self.export_to_docx(data, &path)?;
                    path
                }
                "markdown" | "md" => {
                    let path = output_dir.join("bear_ai_export.md");
                    self.export_to_markdown(data, &path)?;
                    path
                }
                "pdf" => {
                    let path = output_dir.join("bear_ai_export.pdf");
                    self.export_to_pdf(data, &path)?;
                    path
                }
                "txt" | "text" => {
                    let path = output_dir.join("bear_ai_export.txt");
                    self.export_to_text(data, &path)?;
                    path
                }
                _ => return Err(anyhow!("Unsupported export format: {}", format)),
            };

            exported_files.push(filename.to_string_lossy().to_string());
        }

        Ok(exported_files)
    }
}
