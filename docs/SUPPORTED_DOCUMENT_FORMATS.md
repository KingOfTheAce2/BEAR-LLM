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

### Office Documents
- **PDF** - Portable Document Format
  - ✅ Complete text extraction
  - ✅ Multi-page support
  - ✅ Structured content preservation

- **DOCX** - Microsoft Word (2007+)
  - ✅ Complete paragraph & table extraction
  - ✅ Full metadata support
  - ✅ ZIP-based XML parsing

- **DOC** - Microsoft Word 97-2003 (Legacy Format)
  - ✅ Basic text extraction from OLE compound files
  - ✅ Binary format parsing with encoding detection
  - ⚠️ Limited formatting preservation
  - ⚠️ Complex tables may not extract cleanly
  - 📝 Graceful fallback to ASCII text extraction

- **XLSX/XLS** - Microsoft Excel
  - ✅ Complete workbook parsing
  - ✅ All sheets extraction
  - ✅ Cell data with formatting

- **PPTX** - Microsoft PowerPoint (2007+)
  - ✅ Complete slide content extraction
  - ✅ Text and layout parsing
  - ✅ ZIP-based processing

- **PPT** - Microsoft PowerPoint 97-2003 (Legacy Format)
  - ✅ Basic text extraction from OLE compound files
  - ✅ Slide content parsing from binary streams
  - ⚠️ Limited formatting and layout preservation
  - ⚠️ Graphics and embedded objects not extracted
  - 📝 Graceful fallback to ASCII text extraction

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
| DOC    | <400ms      | <2s             | <8s             |
| XLSX   | <400ms      | <2s             | <7s             |
| PPT    | <500ms      | <2.5s           | <10s            |
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
regex = "1"
pdf-extract = "0.7"
calamine = "0.26"      # Excel parsing (XLS/XLSX)
docx-rs = "0.4"        # Word documents (DOCX)
zip = "0.6"            # PPTX/DOCX archives
cfb = "0.9"            # OLE Compound File Binary (DOC/PPT)
encoding_rs = "0.8"    # Character encoding detection
rusqlite = { version = "0.31", features = ["bundled"] }
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
5. **Legacy Formats**: For best results with DOC/PPT files, consider converting to modern formats (DOCX/PPTX) when possible
6. **Format Quality**: Legacy formats (DOC/PPT) provide basic text extraction; complex formatting may not be preserved

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
**Version**: 1.0.5
**Document Formats Supported**: 14 formats with complete implementation

## 📋 Legacy Format Details

### DOC (Microsoft Word 97-2003)
The DOC format uses OLE Compound File Binary structure, which is significantly more complex than modern DOCX. Our implementation provides:

**What Works:**
- Text content extraction from main document stream
- Basic paragraph text recovery
- Multi-encoding support (UTF-8, Windows-1252)
- Graceful degradation to binary text extraction

**Limitations:**
- Complex formatting (bold, italic, colors) not preserved
- Tables extracted as continuous text
- Headers and footers may not be reliably extracted
- Embedded objects and images not processed
- Macros and VBA code not accessible

**Implementation Details:**
- Uses `cfb` crate for OLE file parsing
- Reads `/WordDocument` stream from compound file
- Falls back to printable ASCII extraction if stream parsing fails
- Filters sequences of 3+ printable characters

### PPT (Microsoft PowerPoint 97-2003)
The PPT format also uses OLE Compound File Binary structure with PowerPoint-specific binary records:

**What Works:**
- Text content extraction from slides
- Basic slide text recovery
- Multiple stream parsing (PowerPoint Document, Current User)
- Encoding-aware text extraction

**Limitations:**
- Slide formatting and layout not preserved
- Animations and transitions not processed
- Speaker notes may not be reliably extracted
- Charts and embedded objects not processed
- Slide order may not be perfectly maintained
- Graphics and images not accessible

**Implementation Details:**
- Uses `cfb` crate for OLE file parsing
- Reads `/PowerPoint Document` and `/Current User` streams
- Extracts text from binary record structures
- Falls back to printable text sequences if stream parsing incomplete
- Filters non-textual binary data

### Error Handling
Both DOC and PPT implementations include robust error handling:

1. **Primary Extraction**: Attempt OLE compound file parsing
2. **Fallback Method**: Binary text extraction with encoding detection
3. **Last Resort**: ASCII sequence extraction
4. **User Notification**: Clear error messages when content cannot be extracted

### Recommendations
For mission-critical legal documents:
- **Prefer Modern Formats**: Use DOCX/PPTX when possible for complete fidelity
- **Verify Content**: Always review extracted text for accuracy
- **Keep Originals**: Maintain original DOC/PPT files as source of truth
- **Test First**: Try extraction on sample documents before batch processing