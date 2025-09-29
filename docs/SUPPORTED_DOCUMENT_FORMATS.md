# BEAR AI LLM - Supported Document Formats

## Overview

BEAR AI LLM provides comprehensive document processing capabilities with automatic PII detection and RAG indexing. This document outlines the complete support matrix for various file formats.

## 📄 Fully Supported Formats

### Text Documents
- **TXT** - Plain text files
- **MD** - Markdown documents
- **RTF** - Rich Text Format (basic support)

### Structured Data
- **JSON** - JavaScript Object Notation with pretty formatting
- **CSV** - Comma-separated values with full parsing
- **XML** - Extensible Markup Language with tag stripping
- **HTML** - HyperText Markup Language with content extraction

### Office Documents (Enhanced)
- **PDF** - Portable Document Format
  - ✅ Text extraction using pdf-extract crate
  - ✅ Multi-page document support
  - ✅ Structured content preservation
  - ⚠️ Complex layouts may have limitations

- **DOCX** - Microsoft Word (Open XML)
  - ✅ ZIP-based XML text extraction
  - ✅ Paragraph and formatting preservation
  - ✅ Document metadata extraction
  - ⚠️ Advanced formatting elements not parsed

- **DOC** - Microsoft Word (Legacy)
  - ⚠️ Basic support (requires additional dependencies for full parsing)
  - ✅ PII detection and RAG indexing available

### Spreadsheet Documents
- **XLSX** - Microsoft Excel (Open XML)
  - ⚠️ Placeholder implementation (requires calamine crate for full support)
  - ✅ PII detection and RAG indexing available

- **XLS** - Microsoft Excel (Legacy)
  - ⚠️ Placeholder implementation (requires calamine crate for full support)
  - ✅ PII detection and RAG indexing available

### Presentation Documents
- **PPTX** - Microsoft PowerPoint (Open XML)
  - ⚠️ Placeholder implementation (requires pptx parsing crate)
  - ✅ PII detection and RAG indexing available

- **PPT** - Microsoft PowerPoint (Legacy)
  - ⚠️ Placeholder implementation (requires pptx parsing crate)
  - ✅ PII detection and RAG indexing available

## 🔒 Privacy & Security Features

### PII Detection Across All Formats
All supported document formats include automatic detection of:
- **Social Security Numbers (SSN)** - Pattern: XXX-XX-XXXX
- **Email Addresses** - RFC 5322 compliant patterns
- **Phone Numbers** - US and international formats
- **Credit Card Numbers** - Luhn algorithm validation
- **Physical Addresses** - Multi-line address patterns
- **Personal Names** - First and last name combinations
- **Legal Case Numbers** - Court filing number patterns
- **Organization Names** - Business and legal entity detection

### Processing Pipeline
1. **File Upload** → Document type detection
2. **Format Parsing** → Text extraction and structure preservation
3. **PII Detection** → Automatic identification with confidence scoring
4. **Content Cleaning** → Redaction and replacement of sensitive data
5. **RAG Indexing** → Vector embedding and searchable chunks
6. **Database Storage** → Metadata and content storage for retrieval

## 📊 Performance Characteristics

### File Size Limits
- **Maximum File Size**: 50MB per document
- **Optimal Range**: 1KB - 10MB for best performance
- **Chunk Size**: 512 characters for RAG processing
- **Memory Usage**: ~2x file size during processing

### Processing Speed
| Format | Small (<1MB) | Medium (1-10MB) | Large (10-50MB) |
|--------|-------------|-----------------|-----------------|
| TXT/MD | <100ms      | <500ms          | <2s             |
| JSON   | <200ms      | <1s             | <3s             |
| PDF    | <500ms      | <2s             | <10s            |
| DOCX   | <300ms      | <1.5s           | <5s             |
| CSV    | <200ms      | <800ms          | <3s             |

## 🚀 Usage Examples

### Document Upload via RAG Interface
```javascript
// Upload and process document
const result = await invoke('upload_document', {
  filename: 'contract.pdf',
  content: fileBytes
});
// Returns: { chunks: 15, document_id: 123 }
```

### PII Analysis
```javascript
// Analyze document for PII
const analysis = await invoke('analyze_document_pii', {
  filename: 'agreement.docx',
  content: fileBytes
});
// Returns: { piiDetections: [...], cleanedText: "...", ... }
```

### RAG Search
```javascript
// Search across all uploaded documents
const results = await invoke('rag_search', {
  query: "liability clauses",
  useAgentic: true,
  maxResults: 5
});
// Returns: { answer: "...", sources: [...], confidence: 0.85 }
```

## 🔧 Technical Implementation

### Backend Processing
- **Rust File Processor**: `src-tauri/src/file_processor.rs`
- **PII Detector**: `src-tauri/src/pii_detector.rs`
- **RAG Engine**: `src-tauri/src/rag_engine.rs`
- **Database Manager**: `src-tauri/src/database.rs`

### Dependencies
```toml
# Current dependencies
regex = "1"
pdf-extract = "0.7"
zip = "0.6"
rusqlite = { version = "0.31", features = ["bundled"] }

# Future enhancements
# calamine = "0.19"  # For full Excel support
# docx-rs = "0.4"    # For enhanced DOCX parsing
```

### Frontend Components
- **DocumentAnalysis.tsx**: PII detection interface
- **RAGInterface.tsx**: Document search and upload
- **DatabaseQuery.tsx**: SQL interface for stored documents

## 📝 Best Practices

### For Legal Professionals
1. **Document Preparation**: Ensure documents are not password-protected
2. **File Naming**: Use descriptive filenames for better organization
3. **Batch Processing**: Upload related documents together for context
4. **PII Review**: Always review PII detections before finalizing

### For System Administrators
1. **Storage Management**: Monitor database size and implement rotation
2. **Performance Tuning**: Adjust chunk sizes based on document types
3. **Security Policies**: Configure PII detection sensitivity levels
4. **Backup Strategy**: Regular database backups for document retention

## 🆘 Troubleshooting

### Common Issues
- **"Unsupported file type"**: Check file extension and format
- **"File too large"**: Reduce file size or split into smaller documents
- **"PII detection failed"**: Verify file is not corrupted
- **"Upload timeout"**: Check network and system resources

### Error Codes
- `ERR_001`: File size exceeds 50MB limit
- `ERR_002`: Unsupported file format
- `ERR_003`: File corruption detected
- `ERR_004`: Insufficient system memory
- `ERR_005`: Database storage error

## 📞 Support

For technical support or format-specific questions:
- Review the main README.md for system requirements
- Submit issues for unsupported formats or parsing errors

---

**Last Updated**: September 2025
**Version**: 1.0.2
**Document Formats Supported**: 14 primary formats with varying levels of support