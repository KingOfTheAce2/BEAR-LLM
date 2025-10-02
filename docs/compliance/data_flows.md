# Data Flow Documentation
## BEAR AI LLM - Complete Data Processing Flows

**Document Version:** 1.0
**Last Updated:** 2025-10-02
**Purpose:** Detailed data flow mapping for GDPR Article 30 compliance

---

## Table of Contents

1. [Overview](#overview)
2. [Chat Processing Flow](#chat-processing-flow)
3. [Document Upload Flow](#document-upload-flow)
4. [PII Detection Flow](#pii-detection-flow)
5. [RAG Query Flow](#rag-query-flow)
6. [Consent Management Flow](#consent-management-flow)
7. [Data Export Flow](#data-export-flow)
8. [Retention & Deletion Flow](#retention--deletion-flow)
9. [System Architecture Diagram](#system-architecture-diagram)

---

## Overview

BEAR AI LLM is a **local-first** application with minimal external dependencies. All data flows prioritize:
- **Privacy by Design:** Local processing wherever possible
- **User Control:** Explicit consent and configuration options
- **Transparency:** Clear indication of data processing activities
- **Security:** Encryption, access control, and audit logging

**Primary Data Repositories:**
- SQLite Database (local file system)
- Application Memory (transient, process-bound)
- Local File System (document storage, exports)

**External Services (Optional):**
- HuggingFace API (embedding/inference, opt-in only)

---

## Chat Processing Flow

### 1. User Initiates Chat

```
┌─────────────────────────────────────────────────────────────────┐
│ USER ACTION: Types message in chat interface                   │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: UI Layer (Tauri Frontend)                              │
│ - Captures user input (text string)                            │
│ - Validates input (length, format)                             │
│ - Generates timestamp (ISO 8601)                               │
│ - Assigns message ID (UUID)                                    │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: Consent Check                                          │
│ - Query: ConsentManager::has_consent(user_id, "chat_storage")  │
│ - If NO consent: Show consent dialog, halt processing          │
│ - If YES: Proceed to processing                                │
│ - Log: AuditLogger::log_success("chat_initiated")              │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: PII Detection (Optional)                               │
│ - IF user enabled PII detection:                               │
│   - PIIDetector::detect_pii(message_text)                      │
│   - Returns: Vec<PIIEntity> (types, positions, confidence)     │
│   - User review: Show PII warnings, offer redaction            │
│   - If redacted: Replace with [PII_TYPE] placeholders          │
│ - ELSE: Skip PII detection                                     │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: LLM Processing                                         │
│ - Retrieve context: Last N messages from chat history          │
│ - Construct prompt: System + context + user message            │
│ - LLM inference:                                               │
│   • Local model: gguf_inference.rs (no network)                │
│   • OR Remote API: HuggingFace (if user opted in)              │
│ - Generate response: AI assistant message                      │
│ - Assign response ID: UUID                                     │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 5: Database Storage                                       │
│ - Insert user message:                                         │
│   INSERT INTO chat_messages (chat_id, role='user',             │
│     content, timestamp, metadata)                              │
│ - Insert AI response:                                          │
│   INSERT INTO chat_messages (chat_id, role='assistant',        │
│     content, timestamp, metadata)                              │
│ - Update chat session:                                         │
│   UPDATE chat_sessions SET updated_at=NOW()                    │
│   WHERE id=chat_id                                             │
│ - Set retention:                                               │
│   RetentionManager::set_retention("chat_message", id, 90 days) │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 6: Audit Logging                                          │
│ - AuditLogger::log_success(                                    │
│     user_id: "user_123",                                       │
│     action: DataAccessed,                                      │
│     entity_type: ChatMessage,                                  │
│     entity_id: message_id,                                     │
│     details: {chat_id, model_used, message_length}             │
│   )                                                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 7: Processing Record (GDPR Article 30)                    │
│ - DatabaseManager::log_processing_activity(                    │
│     user_id: "user_123",                                       │
│     processing_purpose: "Chat interaction and AI assistance",  │
│     data_categories: ["user_messages", "ai_responses",         │
│                       "timestamps", "metadata"],               │
│     legal_basis: "Consent (Article 6(1)(a))",                  │
│     retention_days: 90,                                        │
│     recipients: ["local_database", "llm_inference_engine"],    │
│     entity_type: "chat_message",                               │
│     entity_id: message_id                                      │
│   )                                                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ FINAL: Display to User                                         │
│ - Render AI response in chat UI                                │
│ - Show PII warnings if detected                                │
│ - Update chat history panel                                    │
│ - Show "Saved to history" confirmation                         │
└─────────────────────────────────────────────────────────────────┘
```

**Data Elements Processed:**
- **Input:** User message (text), timestamp, chat_id
- **Generated:** AI response, message IDs, PII detections
- **Stored:** Messages, timestamps, metadata, audit logs, processing records
- **Retained:** 90 days (configurable)

**Legal Basis:** Consent (GDPR Article 6(1)(a))

---

## Document Upload Flow

### 2. User Uploads Legal Document

```
┌─────────────────────────────────────────────────────────────────┐
│ USER ACTION: Selects document file (PDF, DOCX, TXT)            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: File Validation                                        │
│ - Check file type: .pdf, .docx, .txt (whitelist)               │
│ - Check file size: < 50MB limit                                │
│ - Virus scan: (TODO - recommend ClamAV integration)            │
│ - Read file content into memory                                │
│ - Extract text:                                                │
│   • PDF: pdf-extract crate                                     │
│   • DOCX: docx-rs crate                                        │
│   • TXT: direct read                                           │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: Consent Check                                          │
│ - ConsentManager::has_consent(user_id, "document_processing")  │
│ - If NO: Show consent dialog with clear explanation            │
│   "Processing this document will enable AI-powered search      │
│    and analysis. Your document will be stored locally."        │
│ - If YES: Proceed                                              │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: PII Detection (CRITICAL PRIVACY STEP)                  │
│ - Run comprehensive PII scan:                                  │
│   • PIIDetector::detect_pii(document_text)                     │
│   • Engines: Presidio (if installed) + Regex fallback          │
│   • Detect: SSN, emails, phones, names, orgs, medical IDs      │
│   • Context enhancement (legal terminology awareness)           │
│ - Present findings to user:                                    │
│   "⚠️  Detected 15 instances of PII:                           │
│     • 3 Social Security Numbers                                │
│     • 5 Person Names                                           │
│     • 7 Email Addresses"                                       │
│ - User options:                                                │
│   [Review PII] [Redact All] [Redact Selected] [Proceed]        │
│ - If redacted: Replace PII with placeholders                   │
│   Original: "John Smith (SSN: 123-45-6789)"                    │
│   Redacted: "[PERSON_001] (SSN: [SSN_001])"                    │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: Document Chunking                                      │
│ - Split document into semantic chunks:                         │
│   • Chunk size: 500-1000 characters                            │
│   • Overlap: 100 characters (preserve context)                 │
│   • Smart splitting: Respect sentence/paragraph boundaries     │
│ - Create chunk records:                                        │
│   Vec<DocumentChunk> {                                         │
│     chunk_index: 0..N,                                         │
│     chunk_text: "...",                                         │
│     document_id: doc_id                                        │
│   }                                                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 5: Vector Embedding                                       │
│ - FOR EACH chunk:                                              │
│   • IF local embeddings:                                       │
│       - Use sentence-transformers (Rust port)                  │
│       - No network required                                    │
│   • ELSE IF HuggingFace enabled (user consented):              │
│       - API call: POST /embeddings                             │
│       - Data sent: Chunk text (over HTTPS)                     │
│       - Receive: 384-dimensional vector                        │
│   • Store: Vector as BLOB in database                          │
│ - Track embedding source in metadata                           │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 6: Database Storage                                       │
│ - Insert document record:                                      │
│   INSERT INTO documents (                                      │
│     filename, content, file_type, upload_date, chunk_count     │
│   ) VALUES (?, ?, ?, NOW(), ?)                                 │
│   RETURNING id as document_id                                  │
│                                                                 │
│ - Insert chunks:                                               │
│   INSERT INTO document_chunks (                                │
│     document_id, chunk_text, chunk_index, vector_embedding     │
│   ) VALUES (?, ?, ?, ?)  -- Batch insert                       │
│                                                                 │
│ - Insert PII detections (metadata only, NO original text):     │
│   INSERT INTO pii_detections (                                 │
│     document_id, pii_type, replacement_text, confidence,       │
│     position_start, position_end                               │
│   ) VALUES (?, ?, ?, ?, ?, ?)                                  │
│                                                                 │
│ - Set retention policy:                                        │
│   UPDATE documents SET retention_until = NOW() + INTERVAL '2 years' │
│   WHERE id = document_id                                       │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 7: Audit & Processing Logs                                │
│ - Audit log:                                                   │
│   AuditLogger::log_success(                                    │
│     action: DataModified,                                      │
│     entity_type: Document,                                     │
│     entity_id: document_id,                                    │
│     details: {filename, chunks, pii_count, embedding_source}   │
│   )                                                            │
│                                                                 │
│ - Processing record:                                           │
│   DatabaseManager::log_processing_activity(                    │
│     purpose: "Legal document analysis and semantic search",    │
│     data_categories: ["document_content", "metadata",          │
│                       "pii_detections", "vector_embeddings"],  │
│     legal_basis: "Consent (Article 6(1)(a))",                  │
│     retention_days: 730 (2 years),                             │
│     recipients: ["local_db", "rag_engine",                     │
│                  "huggingface_api" (if remote)],               │
│     entity_type: "document",                                   │
│     entity_id: document_id                                     │
│   )                                                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ FINAL: User Confirmation                                       │
│ - Show success notification:                                   │
│   "✓ Document processed successfully                           │
│    • 47 chunks created                                         │
│    • 12 PII instances detected and redacted                    │
│    • Ready for semantic search                                 │
│    • Retention: 2 years (expires 2027-10-02)"                  │
│ - Update document library UI                                   │
│ - Enable RAG search for this document                          │
└─────────────────────────────────────────────────────────────────┘
```

**Data Elements Processed:**
- **Input:** Document file (binary), filename, file type
- **Generated:** Document chunks, vector embeddings, PII detections, document ID
- **Stored:** Document content, chunks, embeddings, PII metadata (not original text), audit logs
- **Retained:** 2 years (configurable)

**Legal Basis:** Consent (GDPR Article 6(1)(a))

**Special Considerations:**
- ⚠️ May contain Article 9 special categories (health, criminal records)
- ⚠️ HuggingFace API (third country transfer) only if user opts in
- ✅ PII never stored in original form (only metadata)

---

## PII Detection Flow

### 3. PII Detection Engine

```
┌─────────────────────────────────────────────────────────────────┐
│ INPUT: Text string (from chat or document)                     │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: Engine Selection                                       │
│ - Check Presidio availability:                                 │
│   • Try: python3 -c "import presidio_analyzer"                 │
│   • If success: Use Presidio (enterprise-grade)                │
│   • If fail: Use built-in regex fallback                       │
└────────────────────────────┬────────────────────────────────────┘
                             │
                ┌────────────┴────────────┐
                │                         │
                v                         v
    ┌──────────────────┐      ┌──────────────────┐
    │ PRESIDIO PATH    │      │ REGEX PATH       │
    │ (if available)   │      │ (always runs)    │
    └────────┬─────────┘      └────────┬─────────┘
             │                         │
             v                         v
┌────────────────────────┐  ┌────────────────────────┐
│ Presidio Analysis:     │  │ Regex Detection:       │
│ - NLP-based entity     │  │ - SSN: \d{3}-\d{2}-\d{4}│
│   recognition          │  │ - Email: [a-z@.]+      │
│ - Context-aware        │  │ - Phone: \d{3}-\d{3}-  │
│ - Multi-language       │  │   \d{4}                │
│ - Higher accuracy      │  │ - Credit Card: Luhn    │
│                        │  │   validated            │
│ Returns:               │  │ - Names: Title + Name  │
│ Vec<PIIEntity> {       │  │ - Organizations: Inc/  │
│   type: "EMAIL",       │  │   LLC/LLP patterns     │
│   text: "...",         │  │                        │
│   start: 123,          │  │ Returns:               │
│   end: 145,            │  │ Vec<PIIEntity>         │
│   confidence: 0.95,    │  │                        │
│   engine: "presidio"   │  │                        │
│ }                      │  │                        │
└────────┬───────────────┘  └────────┬───────────────┘
         │                           │
         └───────────┬───────────────┘
                     │
                     v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: Result Merging & Deduplication                         │
│ - Combine results from both engines                            │
│ - Remove duplicates (same position range)                      │
│ - Keep higher confidence score if duplicate                    │
│ - Sort by position (start index)                               │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: Context Enhancement                                    │
│ - Analyze surrounding text (±50 chars) for each detection      │
│ - Boost confidence if context keywords present:                │
│   • "plaintiff", "defendant" → boost PERSON confidence         │
│   • "social security", "SSN" → boost SSN to 1.0                │
│   • "company", "corporation" → boost ORGANIZATION              │
│ - Apply legal term exclusions:                                 │
│   • Load: pii_exclusions.toml                                  │
│   • Exclude: "United States", "Supreme Court", etc.            │
│   • Prevent false positives on legal terminology               │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: Confidence Filtering                                   │
│ - Apply threshold: confidence >= 0.85 (configurable)            │
│ - Filter out low-confidence detections                         │
│ - Final result: Vec<PIIEntity> (high-confidence only)          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ OUTPUT: PII Detection Results                                  │
│ Vec<PIIEntity> {                                               │
│   entity_type: "SSN" | "EMAIL" | "PERSON" | etc.               │
│   text: "[REDACTED - NEVER STORED IN DB]"                      │
│   start: 0..text.len(),                                        │
│   end: 0..text.len(),                                          │
│   confidence: 0.0..1.0,                                        │
│   engine: "presidio" | "regex"                                 │
│ }                                                              │
│                                                                 │
│ ⚠️  CRITICAL: Original PII text NEVER persisted to database    │
│    Only metadata (type, position, confidence) stored           │
└─────────────────────────────────────────────────────────────────┘
```

**Security Measures:**
- ✅ Zero PII storage (metadata only)
- ✅ Dual-engine approach (Presidio + fallback)
- ✅ Legal term exclusions (avoid false positives)
- ✅ Luhn validation for credit cards (prevents false positives)
- ✅ Context-aware boosting (improved accuracy)

---

## RAG Query Flow

### 4. Semantic Search with RAG Engine

```
┌─────────────────────────────────────────────────────────────────┐
│ USER ACTION: Enters search query in RAG search box             │
│ Example: "Find all contract breach clauses"                    │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: Query Embedding                                        │
│ - Convert query to vector:                                     │
│   • IF local: sentence-transformers embedding                  │
│   • IF remote (consented): HuggingFace API call                │
│ - Result: query_vector (384-dim)                               │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: Vector Similarity Search                               │
│ - Query database:                                              │
│   SELECT document_id, chunk_text, chunk_index,                 │
│          cosine_similarity(vector_embedding, ?) as score       │
│   FROM document_chunks                                         │
│   WHERE score > 0.7  -- Similarity threshold                   │
│   ORDER BY score DESC                                          │
│   LIMIT 10  -- Top K results                                   │
│                                                                 │
│ - Cosine similarity calculation:                               │
│   score = (A · B) / (||A|| * ||B||)                            │
│   Where A = query_vector, B = chunk_vector                     │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: Result Ranking & Filtering                             │
│ - Apply relevance filters:                                     │
│   • Minimum similarity: 0.7 (70% match)                        │
│   • Maximum results: 10                                        │
│   • Diversity: Ensure results from multiple documents          │
│ - Enrich results with metadata:                                │
│   • Document filename                                          │
│   • Chunk index (page/section reference)                       │
│   • Upload date                                                │
│   • File type                                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: Context Augmentation (RAG)                             │
│ - Construct augmented prompt:                                  │
│   "Based on the following relevant passages from your          │
│    documents, answer the question:                             │
│                                                                 │
│    [PASSAGE 1 - Contract.pdf, Page 3]                          │
│    '... breach of contract occurs when ...'                    │
│                                                                 │
│    [PASSAGE 2 - LegalGuide.docx, Section 5]                    │
│    '... remedies for breach include ...'                       │
│                                                                 │
│    Question: Find all contract breach clauses                  │
│    Answer:"                                                    │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 5: LLM Generation (with retrieved context)                │
│ - Send augmented prompt to LLM                                 │
│ - LLM generates answer grounded in retrieved passages          │
│ - Reduces hallucinations (real document content)               │
│ - Includes source citations                                    │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 6: Audit & Query Logging                                  │
│ - Log query to history:                                        │
│   INSERT INTO query_history (                                  │
│     query_text, query_type='rag', results_count,               │
│     execution_time_ms, success                                 │
│   )                                                            │
│                                                                 │
│ - Audit log:                                                   │
│   AuditLogger::log_success(                                    │
│     action: DataAccessed,                                      │
│     entity_type: Document,                                     │
│     entity_id: retrieved_document_ids,                         │
│     details: {query, results_count, execution_time}            │
│   )                                                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ FINAL: Display Results to User                                 │
│ - Show AI answer with source citations:                        │
│   "According to your documents, contract breach occurs when... │
│                                                                 │
│    Sources:                                                    │
│    📄 Contract.pdf (Page 3, Similarity: 92%)                   │
│    📄 LegalGuide.docx (Section 5, Similarity: 87%)"            │
│                                                                 │
│ - User can click sources to view full document                 │
│ - Query logged for performance analytics (if consented)        │
└─────────────────────────────────────────────────────────────────┘
```

**Data Flow:**
- **Query** → Embedding → Vector DB → Retrieved Chunks → LLM → Answer
- **No PII exposure:** Chunks are already redacted if user chose redaction
- **Audit trail:** Every document access logged

---

## Consent Management Flow

### 5. User Grants/Revokes Consent

```
┌─────────────────────────────────────────────────────────────────┐
│ USER ACTION: Clicks consent toggle in settings                 │
│ Example: Enable "Chat Storage" consent                         │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: Display Consent Information                            │
│ - Retrieve current consent version:                            │
│   SELECT version, consent_text FROM consent_versions           │
│   WHERE consent_type='chat_storage'                            │
│   AND deprecated_date IS NULL                                  │
│   ORDER BY version DESC LIMIT 1                                │
│                                                                 │
│ - Show consent dialog with full text:                          │
│   "BEAR AI Chat Storage Consent (v2.1)                         │
│                                                                 │
│    By enabling chat storage, you agree to:                     │
│    • Store your messages and AI responses locally              │
│    • Retain chat history for 90 days (configurable)            │
│    • Process messages for PII detection (if enabled)           │
│    • Use data for improving your experience                    │
│                                                                 │
│    You can withdraw consent at any time in Settings."          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: User Decision                                          │
│ - [Grant Consent] → Proceed to STEP 3                          │
│ - [Decline] → No processing, close dialog                      │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: Record Consent Grant                                   │
│ - Get current consent version: v2.1                            │
│ - Get consent text for version                                 │
│ - Insert or update user consent:                               │
│   INSERT INTO user_consent (                                   │
│     user_id, consent_type, granted, granted_at,                │
│     version, consent_text                                      │
│   ) VALUES (?, 'chat_storage', 1, NOW(), 2.1, '...')           │
│   ON CONFLICT (user_id, consent_type, version)                 │
│   DO UPDATE SET granted=1, granted_at=NOW(), revoked_at=NULL   │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: Granular Consent Log (Enhanced Audit)                  │
│ - Capture detailed consent action:                             │
│   INSERT INTO consent_log (                                    │
│     user_id, consent_type, version, granted,                   │
│     ip_address, user_agent, consent_text, timestamp            │
│   ) VALUES (                                                   │
│     'user_123', 'chat_storage', '2.1', true,                   │
│     '192.168.1.100', 'Mozilla/5.0...', '...', NOW()            │
│   )                                                            │
│                                                                 │
│ - This provides immutable proof of consent for compliance      │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 5: Audit Logging                                          │
│ - AuditLogger::log_success(                                    │
│     user_id: "user_123",                                       │
│     action: ConsentGranted,                                    │
│     entity_type: Consent,                                      │
│     entity_id: consent_record_id,                              │
│     details: {                                                 │
│       consent_type: "chat_storage",                            │
│       version: "2.1",                                          │
│       granted_at: "2025-10-02T10:30:00Z",                      │
│       ip_address: "192.168.1.100"                              │
│     }                                                          │
│   )                                                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 6: Enable Feature                                         │
│ - Update application state:                                    │
│   app_state.consents.chat_storage = true                       │
│ - Enable chat storage functionality                            │
│ - Show confirmation:                                           │
│   "✓ Chat storage enabled. Your conversations will be saved."  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ WITHDRAWAL FLOW (User revokes consent later)                   │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: Show withdrawal dialog with reason collection          │
│ - "Are you sure you want to withdraw chat storage consent?     │
│                                                                 │
│    Optional: Tell us why (improves our service):               │
│    [ ] Privacy concerns                                        │
│    [ ] No longer needed                                        │
│    [ ] Switching to different tool                             │
│    [ ] Other: ___________                                      │
│                                                                 │
│    [Cancel] [Withdraw Consent]"                                │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: Record Withdrawal                                      │
│ - Update user_consent:                                         │
│   UPDATE user_consent                                          │
│   SET granted=0, revoked_at=NOW()                              │
│   WHERE user_id=? AND consent_type='chat_storage'              │
│   AND granted=1 AND revoked_at IS NULL                         │
│                                                                 │
│ - Log withdrawal with reason:                                  │
│   INSERT INTO consent_log (                                    │
│     user_id, consent_type, version, granted,                   │
│     withdrawal_reason, ip_address, user_agent                  │
│   ) VALUES (?, 'chat_storage', '2.1', false,                   │
│             'Privacy concerns', ?, ?)                          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: Halt Processing & Notify User                          │
│ - Disable chat storage feature                                 │
│ - Stop storing new messages (existing messages retained)       │
│ - Show notification:                                           │
│   "✓ Consent withdrawn. New chats will not be saved.           │
│      Existing chat history retained per retention policy.      │
│      To delete all chat history, use 'Delete All Data'."       │
└─────────────────────────────────────────────────────────────────┘
```

**GDPR Article 7 Compliance:**
- ✅ Freely given (no service denial if consent withdrawn)
- ✅ Specific (separate consent for each purpose)
- ✅ Informed (full consent text shown)
- ✅ Unambiguous (active opt-in, not pre-checked)
- ✅ Easy withdrawal (one-click, no harder than granting)
- ✅ Proof of consent (immutable audit trail with IP/timestamp)

---

## Data Export Flow (GDPR Article 20)

### 6. User Requests Data Export

```
┌─────────────────────────────────────────────────────────────────┐
│ USER ACTION: Clicks "Export My Data" button in settings        │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: Format Selection                                       │
│ - Show export options:                                         │
│   [ ] JSON (machine-readable, recommended)                     │
│   [ ] Markdown (human-readable)                                │
│   [ ] DOCX (Word document)                                     │
│   [ ] PDF (portable document)                                  │
│   [ ] TXT (plain text)                                         │
│   [✓] All formats                                              │
│                                                                 │
│   Encryption: [x] Password-protect export (recommended)        │
│   Password: [__________]                                       │
│                                                                 │
│   [Cancel] [Generate Export]                                   │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: Data Collection (Parallel Queries)                     │
│ - Query 1: Chat history                                        │
│   SELECT * FROM chat_sessions WHERE user_id=?                  │
│   + JOIN chat_messages ON chat_id                              │
│                                                                 │
│ - Query 2: Document metadata (NOT full content, too large)     │
│   SELECT id, filename, file_type, upload_date, chunk_count,    │
│          pii_detection_count                                   │
│   FROM documents WHERE user_id=?                               │
│                                                                 │
│ - Query 3: PII detections (metadata only)                      │
│   SELECT document_id, pii_type, confidence, position_start,    │
│          position_end, detection_date                          │
│   FROM pii_detections WHERE document_id IN (user_docs)         │
│                                                                 │
│ - Query 4: Consent records                                     │
│   SELECT * FROM user_consent WHERE user_id=?                   │
│   + consent audit trail from consent_log                       │
│                                                                 │
│ - Query 5: Audit logs                                          │
│   SELECT * FROM audit_log WHERE user_id=? LIMIT 1000           │
│                                                                 │
│ - Query 6: User settings                                       │
│   SELECT * FROM user_settings WHERE user_id=?                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: Data Aggregation & Structuring                         │
│ - Build UserDataExport structure:                              │
│   UserDataExport {                                             │
│     export_date: "2025-10-02T10:45:00Z",                       │
│     version: "1.0",                                            │
│     user_id: "user_123",                                       │
│     chats: Vec<ChatExport>,  // All chat sessions + messages   │
│     documents: Vec<DocumentExport>,  // Metadata only          │
│     settings: SettingsExport,                                  │
│     metadata: ExportMetadata {                                 │
│       format_version: "1.0",                                   │
│       application_version: "1.0.25",                           │
│       export_hash: "sha256(...)",  // Integrity verification   │
│       compliance_info: {                                       │
│         gdpr_article_20: true,                                 │
│         encrypted: true (if user set password),                │
│         integrity_verified: true                               │
│       }                                                        │
│     }                                                          │
│   }                                                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: SHA-256 Hash Generation (Integrity)                    │
│ - Serialize data to JSON string                                │
│ - Compute hash: sha256(json_string)                            │
│ - Embed hash in export metadata                                │
│ - User can verify integrity later:                             │
│   "Expected: abc123...                                         │
│    Actual:   abc123...  ✓ Integrity verified"                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 5: Format-Specific Export Generation                      │
│                                                                 │
│ FOR EACH selected format:                                      │
│                                                                 │
│ IF JSON:                                                       │
│   - ExportEngine::export_to_json(data, path)                   │
│   - Direct serialization with serde_json                       │
│   - Includes full structured data                              │
│                                                                 │
│ IF Markdown:                                                   │
│   - ExportEngine::export_to_markdown(data, path)               │
│   - Human-readable format for lawyers                          │
│   - Section headers, bullet points, tables                     │
│   - GDPR compliance statement included                         │
│                                                                 │
│ IF DOCX:                                                       │
│   - ExportEngine::export_to_docx(data, path)                   │
│   - Professional Word document formatting                      │
│   - Title page with compliance statement                       │
│   - Structured sections with headers                           │
│                                                                 │
│ IF PDF:                                                        │
│   - ExportEngine::export_to_pdf(data, path)                    │
│   - Portable document for sharing                              │
│   - Professional layout with metadata                          │
│                                                                 │
│ IF TXT:                                                        │
│   - ExportEngine::export_to_text(data, path)                   │
│   - Plain text fallback                                        │
│   - ASCII art separators for readability                       │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 6: Optional Encryption                                    │
│ - IF user provided password:                                   │
│   - Use AES-256-GCM encryption                                 │
│   - Key derivation: PBKDF2 (100k iterations)                   │
│   - Encrypt all export files                                   │
│   - Add .encrypted extension                                   │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 7: Audit Logging                                          │
│ - AuditLogger::log_success(                                    │
│     action: DataExported,                                      │
│     entity_type: UserSetting,                                  │
│     details: {                                                 │
│       export_formats: ["json", "markdown", "docx"],            │
│       total_chats: 42,                                         │
│       total_documents: 15,                                     │
│       encrypted: true,                                         │
│       export_hash: "abc123..."                                 │
│     }                                                          │
│   )                                                            │
│                                                                 │
│ - Processing record:                                           │
│   DatabaseManager::log_processing_activity(                    │
│     purpose: "GDPR Article 20 Data Portability",               │
│     legal_basis: "Legal Obligation (Article 6(1)(c))",         │
│     data_categories: ["all_user_data"],                        │
│     recipients: ["user"],                                      │
│     retention_days: 30 (export files auto-deleted after 30d)   │
│   )                                                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ FINAL: Deliver Export to User                                  │
│ - Save files to Downloads folder:                              │
│   ~/Downloads/bear_ai_export_2025-10-02/                       │
│     ├── bear_ai_export.json                                    │
│     ├── bear_ai_export.md                                      │
│     ├── bear_ai_export.docx                                    │
│     ├── bear_ai_export.pdf                                     │
│     ├── bear_ai_export.txt                                     │
│     └── README_INTEGRITY.txt (contains SHA-256 hash)           │
│                                                                 │
│ - Show completion dialog:                                      │
│   "✓ Export completed successfully                             │
│                                                                 │
│    Location: ~/Downloads/bear_ai_export_2025-10-02/            │
│    Formats: JSON, Markdown, DOCX, PDF, TXT                     │
│    Integrity Hash: abc123...def456                             │
│    Files will be auto-deleted in 30 days.                      │
│                                                                 │
│    [Open Folder] [Close]"                                      │
│                                                                 │
│ - Schedule cleanup: Add to retention manager (30 days)         │
└─────────────────────────────────────────────────────────────────┘
```

**GDPR Article 20 Compliance:**
- ✅ Structured format (JSON schema)
- ✅ Commonly used (JSON, Markdown, DOCX, PDF)
- ✅ Machine-readable (JSON primary format)
- ✅ Complete data (all user data included)
- ✅ Free of charge (no payment required)
- ✅ Reasonable timeframe (generated on-demand, < 1 minute)
- ✅ Integrity verification (SHA-256 hash)
- ✅ Optional encryption (user privacy)

---

## Retention & Deletion Flow

### 7. Automated Retention Cleanup

```
┌─────────────────────────────────────────────────────────────────┐
│ TRIGGER: Scheduled task runs every 24 hours at 02:00 UTC       │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: Identify Expired Data                                  │
│ - FOR EACH entity type [documents, chat_sessions,              │
│                         chat_messages, query_history]:         │
│                                                                 │
│   SELECT id FROM {entity_type}                                 │
│   WHERE retention_until IS NOT NULL                            │
│   AND retention_until < NOW()                                  │
│                                                                 │
│ - Collect expired IDs for each type                            │
│ - Log retention stats:                                         │
│   - 15 chats expired (older than 90 days)                      │
│   - 3 documents expired (older than 2 years)                   │
│   - 142 query logs expired (older than 30 days)                │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: Cascading Deletion (Preserve Referential Integrity)    │
│                                                                 │
│ FOR documents:                                                 │
│   - Delete related chunks:                                     │
│     DELETE FROM document_chunks                                │
│     WHERE document_id IN (expired_doc_ids)                     │
│   - Delete related PII detections:                             │
│     DELETE FROM pii_detections                                 │
│     WHERE document_id IN (expired_doc_ids)                     │
│   - Delete document record:                                    │
│     DELETE FROM documents                                      │
│     WHERE id IN (expired_doc_ids)                              │
│                                                                 │
│ FOR chat_sessions:                                             │
│   - Delete related messages:                                   │
│     DELETE FROM chat_messages                                  │
│     WHERE chat_id IN (expired_chat_ids)                        │
│   - Delete session record:                                     │
│     DELETE FROM chat_sessions                                  │
│     WHERE id IN (expired_chat_ids)                             │
│                                                                 │
│ FOR query_history:                                             │
│   - Direct deletion (no dependencies):                         │
│     DELETE FROM query_history                                  │
│     WHERE id IN (expired_query_ids)                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: Secure Deletion (VACUUM)                               │
│ - SQLite VACUUM command:                                       │
│   VACUUM;  -- Reclaim space, overwrite deleted data            │
│                                                                 │
│ - This ensures deleted data is not recoverable from disk       │
│ - File system blocks overwritten                               │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: Audit Logging                                          │
│ - Log retention cleanup completion:                            │
│   AuditLogger::log_success(                                    │
│     user_id: "system",                                         │
│     action: SettingChanged,                                    │
│     entity_type: UserSetting,                                  │
│     details: {                                                 │
│       action: "retention_cleanup",                             │
│       documents_deleted: 3,                                    │
│       chats_deleted: 15,                                       │
│       queries_deleted: 142,                                    │
│       space_reclaimed_mb: 47.3                                 │
│     }                                                          │
│   )                                                            │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             v
┌─────────────────────────────────────────────────────────────────┐
│ STEP 5: User Notification (Optional)                           │
│ - IF user opted for retention notifications:                   │
│   - Show notification on next app launch:                      │
│     "🗑️ Retention Cleanup Report                               │
│      • 15 old chats deleted (older than 90 days)               │
│      • 3 documents deleted (older than 2 years)                │
│      • 47.3 MB disk space reclaimed                            │
│                                                                 │
│      [View Details] [Adjust Retention] [OK]"                   │
└─────────────────────────────────────────────────────────────────┘
```

**Deletion Guarantees:**
- ✅ Cascading deletion (no orphaned records)
- ✅ VACUUM for secure deletion (no disk recovery)
- ✅ Audit trail of all deletions
- ✅ User notification (optional)

---

## System Architecture Diagram

```
┌───────────────────────────────────────────────────────────────────┐
│                        BEAR AI LLM SYSTEM                         │
│                      (Local-First Privacy)                        │
└───────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ PRESENTATION LAYER (Tauri + React)                             │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  Chat UI     │  │  Doc Manager │  │  Settings    │         │
│  │  Component   │  │  Component   │  │  Component   │         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│         │                 │                 │                   │
└─────────┼─────────────────┼─────────────────┼───────────────────┘
          │                 │                 │
          v                 v                 v
┌─────────────────────────────────────────────────────────────────┐
│ APPLICATION LAYER (Tauri Commands - Rust)                      │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │ LLM Manager  │  │ File Proc.   │  │ Compliance   │         │
│  │ (gguf_infer) │  │ (processor)  │  │ Manager      │         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│         │                 │                 │                   │
│         │                 │                 │                   │
│  ┌──────┴───────┐  ┌──────┴───────┐  ┌──────┴───────┐         │
│  │ RAG Engine   │  │ PII Detector │  │ Export Engine│         │
│  │ (embeddings) │  │ (Presidio)   │  │ (multi-fmt)  │         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│         │                 │                 │                   │
└─────────┼─────────────────┼─────────────────┼───────────────────┘
          │                 │                 │
          v                 v                 v
┌─────────────────────────────────────────────────────────────────┐
│ PERSISTENCE LAYER                                               │
│                                                                 │
│  ┌─────────────────────────────────────────────────┐           │
│  │     SQLite Database (Local File System)         │           │
│  │                                                  │           │
│  │  ┌────────────┐  ┌────────────┐  ┌──────────┐  │           │
│  │  │ Documents  │  │ Chats      │  │ Consents │  │           │
│  │  │ Chunks     │  │ Messages   │  │ Audit    │  │           │
│  │  │ PII Meta   │  │ Embeddings │  │ Settings │  │           │
│  │  └────────────┘  └────────────┘  └──────────┘  │           │
│  │                                                  │           │
│  │  Features:                                       │           │
│  │  • Connection Pooling (max 5)                   │           │
│  │  • Encryption at Rest (SQLCipher - TODO)        │           │
│  │  • Automated Retention Cleanup (daily)          │           │
│  │  • VACUUM for Secure Deletion                   │           │
│  └─────────────────────────────────────────────────┘           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ EXTERNAL SERVICES (Optional, User-Controlled)                  │
│                                                                 │
│  ┌──────────────────────────────────────┐                      │
│  │  HuggingFace API (USA)               │                      │
│  │  • Embeddings (if remote enabled)    │                      │
│  │  • Inference (if remote enabled)     │                      │
│  │  • HTTPS encryption                  │                      │
│  │  • SCCs for GDPR compliance          │                      │
│  │  • Opt-in only (disabled by default) │                      │
│  └──────────────────────────────────────┘                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ DATA FLOW PRINCIPLES                                            │
│                                                                 │
│  1️⃣  Local First: All data stored locally by default           │
│  2️⃣  Consent Gated: External services require explicit consent │
│  3️⃣  PII Protected: Detection & redaction before storage       │
│  4️⃣  Audit Logged: Every data operation tracked               │
│  5️⃣  Retention Enforced: Automated cleanup per policy          │
│  6️⃣  Export Ready: GDPR Article 20 on-demand exports           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

**Document Control:**
- Version: 1.0
- Status: Active
- Next Review: 2026-01-02
- Approved By: [System Architect]
- Date: 2025-10-02
