# GDPR Article 30 - Record of Processing Activities
## BEAR AI LLM - Legal Document Assistant

**Document Version:** 1.0
**Last Updated:** 2025-10-02
**Data Controller:** BEAR AI Development Team
**Contact:** privacy@bear-ai.local
**Compliance Officer:** TBD

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Legal Framework](#legal-framework)
3. [Processing Activities Overview](#processing-activities-overview)
4. [Detailed Processing Records](#detailed-processing-records)
5. [Third-Party Processors](#third-party-processors)
6. [Data Subject Rights](#data-subject-rights)
7. [Review and Maintenance](#review-and-maintenance)

---

## Executive Summary

This document constitutes the Record of Processing Activities as required by **GDPR Article 30**. BEAR AI LLM is a local-first AI legal assistant that processes personal data for document analysis, chat interactions, and AI-powered legal research.

**Key Processing Principles:**
- **Privacy by Design**: All processing designed with data protection from inception
- **Local-First**: Primary processing occurs locally, minimizing third-party data exposure
- **Consent-Based**: Granular consent mechanisms for all optional processing activities
- **Retention-Controlled**: Automated retention policies with configurable periods
- **Audit Trail**: Comprehensive logging of all data processing activities

---

## Legal Framework

### Applicable Regulations
- **GDPR** (EU Regulation 2016/679) - General Data Protection Regulation
- **EU AI Act** (Regulation 2024/1689) - Artificial Intelligence Act
- **National Data Protection Laws** - As applicable in user jurisdiction

### Data Controller Information
- **Name:** BEAR AI - Legal Document Assistant
- **Type:** Software Application (Open Source)
- **Contact:** privacy@bear-ai.local
- **DPO (if applicable):** TBD
- **Representative (if applicable):** TBD

---

## Processing Activities Overview

| Activity ID | Purpose | Legal Basis | Data Categories | Retention |
|-------------|---------|-------------|-----------------|-----------|
| PA-001 | Chat Storage & History | Consent (Art. 6(1)(a)) | User messages, timestamps, model responses | 90 days (configurable) |
| PA-002 | Document Processing & Embedding | Consent (Art. 6(1)(a)) | Document content, metadata, embeddings | 2 years (configurable) |
| PA-003 | PII Detection & Anonymization | Legitimate Interest (Art. 6(1)(f)) | Detected PII types, confidence scores, positions | Linked to document retention |
| PA-004 | User Consent Management | Legal Obligation (Art. 6(1)(c)) | Consent records, versions, timestamps, IP addresses | 7 years (legal requirement) |
| PA-005 | Audit Logging | Legal Obligation (Art. 6(1)(c)) | User actions, timestamps, IP addresses, outcomes | 2 years |
| PA-006 | Data Export Generation | Legal Obligation (Art. 6(1)(c)) | All user data in portable format | On-demand, 30 days retention |
| PA-007 | Query History & Analytics | Consent (Art. 6(1)(a)) | Search queries, execution times, results | 30 days |
| PA-008 | User Settings & Preferences | Contract Performance (Art. 6(1)(b)) | Application preferences, UI state | Duration of use |

---

## Detailed Processing Records

### PA-001: Chat Storage & History

**Processing Purpose:**
Store and retrieve conversation history between user and AI assistant to provide context-aware responses and enable users to review past interactions.

**Legal Basis:**
- **Primary:** Consent (GDPR Article 6(1)(a))
- **Secondary:** Contract Performance (Article 6(1)(b)) - necessary for service delivery

**Categories of Data Processed:**
- User messages (text content)
- AI assistant responses
- Timestamps (message creation/update)
- Chat session metadata (title, model used, tags)
- Message role indicators (user/assistant)

**Data Subjects:**
- Individual users of BEAR AI application
- Legal professionals, researchers, students

**Recipients:**
- **Internal:** Local SQLite database, application memory
- **External:** None (unless user explicitly exports data)

**Third-Country Transfers:**
- **None** - All data stored locally on user's device

**Retention Period:**
- **Default:** 90 days from last message
- **Configurable:** User can set retention from 7 days to indefinite
- **Automated Deletion:** Via retention manager (`RetentionManager::delete_expired_entities`)

**Security Measures:**
- Local database encryption (SQLite native encryption)
- File system permissions (OS-level access control)
- PII detection and optional redaction
- Audit logging of all access
- Secure deletion via retention policies

**Data Flow:**
```
User Input → Chat UI → LLM Processing → Database Storage
             ↓
        PII Detection (optional)
             ↓
        Consent Check → Audit Log
             ↓
        Retention Policy Applied
```

**Implementation References:**
- Database table: `chat_sessions`, `chat_messages`
- Code: `/src-tauri/src/database.rs` (lines 188-212)
- Retention: `/src-tauri/src/compliance/retention.rs`
- Consent: `/src-tauri/src/compliance/consent.rs`

---

### PA-002: Document Processing & Embedding

**Processing Purpose:**
Process uploaded legal documents to enable semantic search, RAG (Retrieval-Augmented Generation), and AI-powered document analysis.

**Legal Basis:**
- **Primary:** Consent (GDPR Article 6(1)(a))
- **Secondary:** Legitimate Interest (Article 6(1)(f)) - improving legal research capabilities

**Categories of Data Processed:**
- Document content (full text)
- Document metadata (filename, file type, upload date)
- Document chunks (segmented text for embedding)
- Vector embeddings (numerical representations)
- PII detections (type, confidence, position - NOT original text)
- Chunk count and indexing information

**Data Subjects:**
- Document authors (may include clients, opposing parties, witnesses)
- Legal professionals uploading documents
- Referenced individuals in legal documents

**Special Categories (Article 9):**
- **Potential:** Legal documents may contain health data, criminal records, or other sensitive information
- **Protection:** Automatic PII detection and redaction capabilities
- **User Control:** Optional anonymization before processing

**Recipients:**
- **Internal:** Local SQLite database, RAG engine, embedding models
- **External:** HuggingFace API (if remote embedding models used with user consent)

**Third-Country Transfers:**
- **HuggingFace API (USA):** Only if user enables remote embedding models
- **Safeguards:** Standard Contractual Clauses (SCCs), user consent required
- **Alternative:** Local embedding models (no third-country transfer)

**Retention Period:**
- **Default:** 2 years from upload date
- **Configurable:** User can set retention or keep indefinitely
- **Cascading Deletion:** Related chunks and PII detections deleted with document

**Security Measures:**
- PII detection before storage (Presidio + regex patterns)
- Encryption at rest (database level)
- Access control (application-level authentication)
- Secure deletion with VACUUM operation
- Audit trail for all document access
- Vector embedding obfuscation (no reverse-engineering to original text)

**Data Flow:**
```
Document Upload → File Validation → Content Extraction
                       ↓
                 PII Detection → User Review → Redaction (optional)
                       ↓
                 Chunking → Vector Embedding → Database Storage
                       ↓
                 Consent Check → Processing Record → Audit Log
                       ↓
                 Retention Policy Applied
```

**Implementation References:**
- Database tables: `documents`, `document_chunks`, `pii_detections`
- Code: `/src-tauri/src/database.rs` (lines 102-125, 341-365)
- PII Detection: `/src-tauri/src/pii_detector.rs`
- RAG Engine: `/src-tauri/src/rag_engine.rs`
- Retention: `/src-tauri/src/compliance/retention.rs` (lines 154-205)

---

### PA-003: PII Detection & Anonymization

**Processing Purpose:**
Automatically detect and optionally redact Personally Identifiable Information (PII) to protect privacy and comply with data minimization principles.

**Legal Basis:**
- **Primary:** Legitimate Interest (GDPR Article 6(1)(f)) - protecting data subject privacy
- **Balancing Test:** Detection benefits outweigh processing (non-intrusive, protective measure)

**Categories of Data Processed:**
- PII type classifications (SSN, email, phone, name, etc.)
- Confidence scores (detection accuracy 0.0-1.0)
- Position information (start/end character indices)
- Replacement text (anonymized placeholders)
- Detection timestamps
- Document references (which document contains PII)

**CRITICAL SECURITY MEASURE:**
**Original PII text is NEVER stored in the database.** Only metadata about detections is retained.

**PII Types Detected:**
1. **Identifiers:**
   - Social Security Numbers (SSN)
   - Medical Record Numbers (MRN)
   - Case Numbers (legal documents)
   - Credit Card Numbers (Luhn-validated)

2. **Contact Information:**
   - Email addresses
   - Phone numbers (multiple formats)
   - IP addresses

3. **Named Entities:**
   - Person names (with context awareness)
   - Organizations (companies, law firms)
   - Locations (when sensitive)

**Detection Engines:**
- **Primary:** Microsoft Presidio (if installed) - enterprise-grade NLP
- **Fallback:** Regex patterns + Luhn validation - built-in reliability
- **Context Enhancement:** Surrounding text analysis for improved accuracy

**Recipients:**
- **Internal:** PII detection service, database
- **External:** Microsoft Presidio (local Python process, no cloud)

**Third-Country Transfers:**
- **None** - All PII detection occurs locally

**Retention Period:**
- **Linked to Document:** PII records deleted when parent document is deleted
- **Cascading Deletion:** Automatic cleanup via foreign key constraints

**Security Measures:**
- **Zero PII Storage:** Original PII text never persisted
- Only metadata stored (type, position, confidence)
- Presidio runs locally (no cloud transmission)
- Fallback patterns for reliability
- Exclusion lists for legal terms (avoid false positives)
- Audit logging of all detections

**Data Flow:**
```
Document/Chat Input → PII Detector (Presidio + Regex)
                           ↓
                    Detection Results → Context Enhancement
                           ↓
                    Confidence Filtering (>0.85)
                           ↓
                    Metadata Only → Database (NO original PII text)
                           ↓
                    User Notification → Optional Redaction
```

**Implementation References:**
- Code: `/src-tauri/src/pii_detector.rs` (complete implementation)
- Database: `/src-tauri/src/database.rs` (lines 368-395)
- Schema: Lines 128-142 (pii_detections table)
- Exclusions: `pii_exclusions.toml` configuration

**Privacy Impact Assessment:**
- **Risk:** Low - protective measure, no PII stored
- **Benefit:** High - prevents accidental PII exposure
- **Mitigation:** Exclusion lists prevent over-flagging legal terms

---

### PA-004: User Consent Management

**Processing Purpose:**
Record and manage user consent for various data processing activities to comply with GDPR consent requirements.

**Legal Basis:**
- **Legal Obligation (GDPR Article 6(1)(c))** - Required to demonstrate compliance

**Categories of Data Processed:**
- User identifier
- Consent type (chat_storage, document_processing, pii_detection, etc.)
- Consent status (granted/revoked)
- Consent version number
- Consent text shown to user
- Grant timestamp (ISO 8601 format)
- Revocation timestamp (if applicable)
- IP address (for verification)
- User agent (for audit purposes)
- Withdrawal reason (if provided)

**Consent Types Managed:**
1. `pii_detection` - PII detection and anonymization
2. `chat_storage` - Chat history retention
3. `document_processing` - Document analysis and embedding
4. `analytics` - Usage analytics (if implemented)
5. `ai_processing` - AI model processing
6. `data_retention` - Automated retention policies

**Recipients:**
- **Internal:** Consent database, audit logger
- **External:** None

**Third-Country Transfers:**
- **None** - All consent data stored locally

**Retention Period:**
- **7 years** - Legal requirement for consent records
- **Post-withdrawal:** Consent history retained for legal defense (legitimate interest)

**Security Measures:**
- Immutable audit trail (consent history never deleted, only updated)
- Granular consent log with IP addresses and user agents
- Version control for consent text changes
- Easy withdrawal mechanism (GDPR Article 7(3))
- Timestamp verification
- Database integrity constraints

**Consent Workflow:**
```
User Action (Grant/Revoke) → Consent Manager
                 ↓
         Version Validation → Current Consent Text Retrieved
                 ↓
         Database Update (user_consent table)
                 ↓
         Granular Log Entry (consent_log table)
                 ↓
         Audit Logger → Processing Record
                 ↓
         User Notification (confirmation)
```

**Withdrawal Mechanism:**
- One-click withdrawal from UI
- Withdrawal reason optional but encouraged
- Immediate processing halt for revoked consent
- Audit trail of withdrawal maintained
- User notification of consequences (e.g., feature unavailability)

**Implementation References:**
- Code: `/src-tauri/src/compliance/consent.rs` (complete implementation)
- Database tables: `user_consent`, `consent_versions`, `consent_log`
- Migrations: `001_create_user_consent.sql`, `002_create_consent_versions.sql`, `006_create_consent_log.sql`
- API: `ConsentManager` struct (lines 72-479)

**GDPR Article 7 Compliance:**
- ✅ Free withdrawal at any time (line 405-430)
- ✅ Easy as giving consent (single method call)
- ✅ Burden of proof on controller (complete audit trail)
- ✅ Informed consent (full consent text stored)
- ✅ Granular consent (separate consent types)

---

### PA-005: Audit Logging

**Processing Purpose:**
Comprehensive logging of all data processing activities to ensure accountability, detect security incidents, and demonstrate GDPR compliance.

**Legal Basis:**
- **Legal Obligation (GDPR Article 6(1)(c))** - Required for compliance demonstration
- **Legitimate Interest (Article 6(1)(f))** - Security and fraud detection

**Categories of Data Processed:**
- User identifier
- Action type (consent_granted, data_accessed, data_exported, etc.)
- Entity type (document, chat_message, consent, user_setting)
- Entity identifier (specific record ID)
- Timestamp (ISO 8601 format with millisecond precision)
- Success/failure status
- Error messages (if failure)
- IP address (optional, for security)
- User agent (optional, for security)
- Additional metadata (JSON format, context-specific)

**Audit Actions Logged:**
1. **Consent Actions:**
   - `consent_granted` - User grants consent
   - `consent_revoked` - User withdraws consent

2. **Data Access:**
   - `data_accessed` - User views data
   - `data_modified` - User edits data
   - `data_exported` - User exports data (GDPR Article 20)
   - `data_deleted` - User deletes data (GDPR Article 17)

3. **User Actions:**
   - `user_login` - Authentication event
   - `user_logout` - Session termination
   - `setting_changed` - Preference update

**Recipients:**
- **Internal:** Audit database, compliance manager
- **External:** None

**Third-Country Transfers:**
- **None** - All audit logs stored locally

**Retention Period:**
- **Default:** 2 years from log entry
- **Rationale:** Balance between accountability and storage minimization
- **Cleanup:** Automated deletion via `AuditLogger::delete_old_logs(730)`

**Security Measures:**
- Append-only logging (no modification of existing logs)
- Database integrity constraints
- Indexed queries for performance
- Encrypted storage (database level)
- Access control (compliance officer only for exports)
- Automated retention cleanup

**Data Flow:**
```
User Action → Application Layer → Audit Logger
                    ↓
            Log Entry Creation → Timestamp + User Context
                    ↓
            Database Write (audit_log table)
                    ↓
            Success Confirmation → Application Continues
                    ↓
            Periodic Cleanup (every 24h) → Old Logs Deleted
```

**Query and Reporting:**
- User-specific logs (GDPR transparency)
- Entity-specific logs (document access history)
- Date range filtering
- Action type filtering
- Success/failure statistics
- Search by keyword in metadata

**Implementation References:**
- Code: `/src-tauri/src/compliance/audit.rs` (complete implementation)
- Database table: `audit_log`
- Migration: `004_create_audit_log.sql`
- API: `AuditLogger` struct (lines 102-389)

**GDPR Article 5(2) Compliance:**
- ✅ Accountability - Complete audit trail
- ✅ Transparency - User can access own logs
- ✅ Integrity - Append-only, tamper-evident
- ✅ Confidentiality - Access controlled

---

### PA-006: Data Export Generation (GDPR Article 20)

**Processing Purpose:**
Generate comprehensive, machine-readable exports of all user data to fulfill the right to data portability.

**Legal Basis:**
- **Legal Obligation (GDPR Article 6(1)(c))** - Required by Article 20

**Categories of Data Processed:**
- All user data from other processing activities:
  - Chat history (all messages)
  - Documents (metadata, NOT full content due to size)
  - PII detections (metadata only)
  - Consent records (full audit trail)
  - Audit logs (user-specific)
  - User settings and preferences
- Export metadata:
  - Export timestamp
  - Export hash (SHA-256 for integrity)
  - Export format version
  - Application version

**Export Formats Supported:**
1. **JSON** - Primary machine-readable format (GDPR requirement)
2. **Markdown** - Human-readable, lawyer-friendly format
3. **DOCX** - Professional Word document with legal formatting
4. **PDF** - Portable document for sharing
5. **TXT** - Plain text fallback

**Recipients:**
- **User** - Receives export via local file system
- **Internal:** Export engine, database
- **External:** None (unless user manually transmits export)

**Third-Country Transfers:**
- **None** - Export generated locally, user controls distribution

**Retention Period:**
- **On-demand generation** - No persistent storage of exports
- **Temporary files:** Deleted after user downloads (30 days max)
- **User responsibility:** User controls export file after download

**Security Measures:**
- SHA-256 integrity hash included in export
- Optional encryption (user-provided password)
- Comprehensive GDPR compliance statement in export
- Structured format prevents accidental data loss
- PII redaction options before export
- Audit logging of export requests

**Export Data Structure:**
```json
{
  "export_date": "ISO 8601 timestamp",
  "version": "1.0",
  "user_id": "user_identifier",
  "metadata": {
    "format_version": "1.0",
    "application_version": "1.0.25",
    "export_hash": "sha256_hash",
    "compliance_info": {
      "gdpr_article_20": true,
      "encrypted": false,
      "integrity_verified": true
    }
  },
  "chats": [ /* ChatExport objects */ ],
  "documents": [ /* DocumentExport metadata */ ],
  "settings": { /* User preferences */ }
}
```

**Data Flow:**
```
User Request → Compliance Manager → Data Collection
                    ↓
            Database Queries (chats, docs, consents, audit)
                    ↓
            Data Aggregation → Export Engine
                    ↓
            Format Selection (JSON/MD/DOCX/PDF/TXT)
                    ↓
            Hash Generation (SHA-256)
                    ↓
            File Creation → User Download
                    ↓
            Audit Log Entry → Cleanup Scheduling
```

**Implementation References:**
- Code: `/src-tauri/src/export_engine.rs` (complete implementation)
- Compliance: `/src-tauri/src/compliance/mod.rs` (lines 160-193)
- Data structures: Lines 12-79 (export types)
- Formats: Lines 96-478 (DOCX, Markdown, PDF, Text exporters)

**GDPR Article 20 Compliance:**
- ✅ Structured format - JSON with clear schema
- ✅ Commonly used - Standard JSON, Markdown, DOCX, PDF
- ✅ Machine-readable - JSON parseable by any system
- ✅ Complete data - All user data included
- ✅ No fee - Export is free
- ✅ Reasonable timeframe - Generated on-demand (< 1 minute)

---

### PA-007: Query History & Analytics

**Processing Purpose:**
Track search queries and system performance for quality improvement, debugging, and user experience enhancement.

**Legal Basis:**
- **Consent (GDPR Article 6(1)(a))** - User opts in to analytics
- **Legitimate Interest (Article 6(1)(f))** - System debugging and improvement

**Categories of Data Processed:**
- Query text (search terms used)
- Query type (rag, sql, agentic)
- Results count (number of results returned)
- Execution time (milliseconds)
- Success/failure status
- Error messages (if query failed)
- Query timestamp

**Data Subjects:**
- Individual users of BEAR AI application

**Recipients:**
- **Internal:** Database, analytics module
- **External:** None

**Third-Country Transfers:**
- **None** - All query data stored locally

**Retention Period:**
- **Default:** 30 days from query execution
- **Rationale:** Short retention for recent performance analysis
- **Automated deletion:** Via retention manager

**Security Measures:**
- Query text sanitized (no PII in search logs)
- Limited retention period (30 days)
- No user linking beyond session
- Aggregated analytics (no individual profiling)
- Audit trail of query access

**Data Flow:**
```
User Search → RAG/SQL Engine → Query Execution
                    ↓
            Result Collection → Performance Metrics
                    ↓
            Database Logging (query_history table)
                    ↓
            Consent Check → Analytics Processing (if consented)
                    ↓
            Retention Policy (30 days) → Automated Deletion
```

**Implementation References:**
- Code: `/src-tauri/src/database.rs` (lines 462-508)
- Database table: `query_history` (lines 173-185)
- Analytics: Aggregated statistics, no individual profiling

---

### PA-008: User Settings & Preferences

**Processing Purpose:**
Store user preferences and application configuration to provide personalized experience and remember user choices.

**Legal Basis:**
- **Contract Performance (GDPR Article 6(1)(b))** - Necessary for service delivery
- **Legitimate Interest (Article 6(1)(f))** - Improving user experience

**Categories of Data Processed:**
- Application preferences (theme, language, display options)
- UI state (window size, panel layout)
- Default settings (retention periods, AI models)
- Feature flags (enabled/disabled features)
- Last updated timestamp

**Data Subjects:**
- Individual users of BEAR AI application

**Recipients:**
- **Internal:** Database, application state manager
- **External:** None

**Third-Country Transfers:**
- **None** - All settings stored locally

**Retention Period:**
- **Duration of use** - Settings retained while user uses application
- **Post-uninstall:** User can choose to delete or preserve settings
- **Automated cleanup:** Optional, user-controlled

**Security Measures:**
- Local database storage
- No sensitive data in settings
- User-controlled deletion
- Encrypted storage (database level)

**Data Flow:**
```
User Configuration → UI Layer → Settings Manager
                        ↓
                Database Write (user_settings table)
                        ↓
                Application State Update
                        ↓
                User Confirmation
```

**Implementation References:**
- Code: `/src-tauri/src/database.rs` (lines 215-223)
- Database table: `user_settings`

---

## Third-Party Processors

### HuggingFace API (Optional, User-Controlled)

**Purpose:** Remote AI model inference and embeddings (only if user enables)

**Data Processed:**
- Document text (only if remote embeddings enabled)
- Query text (for semantic search)

**Location:** United States (Cloudflare CDN globally)

**Safeguards:**
- Standard Contractual Clauses (SCCs)
- User consent required before any transmission
- Local-first alternative always available
- HTTPS encryption in transit
- HuggingFace privacy policy compliance

**Data Processing Agreement:** [HuggingFace Terms](https://huggingface.co/terms-of-service)

**User Control:**
- Opt-in only (disabled by default)
- Can switch to local models anytime
- Clear indication when remote API is used

### Microsoft Presidio (Optional, Local Only)

**Purpose:** Enhanced PII detection (local Python process)

**Data Processed:**
- Document text (local processing only)
- Chat messages (if PII detection enabled)

**Location:** User's local machine (no cloud transmission)

**Safeguards:**
- Runs entirely locally (no network access)
- Fallback to built-in detection if not installed
- User controls whether to use Presidio

**Data Processing Agreement:** Not applicable (local tool, no data processor relationship)

---

## Data Subject Rights

BEAR AI implements comprehensive support for all GDPR data subject rights:

### Right to Access (Article 15)
- **Implementation:** Export function provides complete data access
- **Format:** JSON (machine-readable) + human-readable formats
- **Timeframe:** Instant (on-demand generation)

### Right to Rectification (Article 16)
- **Implementation:** Users can edit all stored data via UI
- **Audit:** Changes logged in audit trail

### Right to Erasure (Article 17)
- **Implementation:** `ComplianceManager::delete_user_data()`
- **Scope:** Complete data deletion including cascaded relationships
- **Verification:** Audit log entry created

### Right to Restriction (Article 18)
- **Implementation:** Consent withdrawal mechanism
- **Effect:** Processing halted for specific consent types
- **Retention:** Data marked for deletion, not processed

### Right to Data Portability (Article 20)
- **Implementation:** `ExportEngine` with multiple formats
- **Formats:** JSON, Markdown, DOCX, PDF, TXT
- **Completeness:** All user data included

### Right to Object (Article 21)
- **Implementation:** Granular consent management
- **Scope:** Object to specific processing activities
- **Effect:** Immediate processing cessation

### Automated Decision-Making (Article 22)
- **Not Applicable:** BEAR AI is an assistant tool, all decisions made by user
- **Human Oversight:** User controls all AI interactions

**Implementation Reference:**
- Code: `/src-tauri/src/compliance/mod.rs` (lines 160-226)

---

## Review and Maintenance

### Regular Review Schedule
- **Quarterly:** Review processing activities for changes
- **Annually:** Full compliance audit and risk assessment
- **Ad-hoc:** Review triggered by new features or regulation changes

### Document Updates
- **Version Control:** All changes tracked in git
- **Change Log:** Maintained in this document
- **Notification:** Users informed of material changes

### Compliance Monitoring
- **Automated:** Audit log analysis for anomalies
- **Manual:** Periodic review of consent statistics
- **Incident Response:** Breach notification procedures (< 72 hours)

### Contact for Questions
- **Email:** privacy@bear-ai.local
- **Issue Tracker:** https://github.com/[repo]/issues
- **Documentation:** https://bear-ai.local/docs/compliance

---

## Appendix A: Data Flow Diagram (Textual)

```
┌─────────────┐
│    USER     │
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       v                 v
┌─────────────┐   ┌──────────────┐
│  Chat UI    │   │ Doc Upload   │
└──────┬──────┘   └──────┬───────┘
       │                 │
       v                 v
┌─────────────┐   ┌──────────────┐
│ Consent     │   │ PII Detector │
│ Check       │   │ (Presidio)   │
└──────┬──────┘   └──────┬───────┘
       │                 │
       ├─────────────────┤
       v                 v
┌──────────────────────────┐
│   Local SQLite DB        │
│   (Encrypted at Rest)    │
└────────┬─────────────────┘
         │
         ├───────┬───────┬────────┐
         v       v       v        v
    ┌────────┐ ┌────┐ ┌────┐ ┌──────┐
    │ Chats  │ │Docs│ │PII │ │Audit │
    └────────┘ └────┘ └────┘ └──────┘
         │       │       │        │
         └───────┴───────┴────────┘
                   │
                   v
         ┌──────────────────┐
         │ Retention Manager│
         │ (Auto-Cleanup)   │
         └──────────────────┘
                   │
                   v
         ┌──────────────────┐
         │ Export Engine    │
         │ (GDPR Art. 20)   │
         └──────────────────┘
                   │
                   v
              ┌────────┐
              │  USER  │
              └────────┘
```

---

## Appendix B: Consent Matrix

| Feature | Consent Type | Required | Default | Withdrawal Impact |
|---------|--------------|----------|---------|-------------------|
| Chat History | `chat_storage` | No | Yes | Past chats retained, future not stored |
| Document Upload | `document_processing` | Yes | Yes | Cannot process documents |
| PII Detection | `pii_detection` | No | Yes | PII not detected/redacted |
| Analytics | `analytics` | No | No | No usage stats collected |
| Remote Embeddings | `ai_processing` | No | No | Must use local models |

---

**Document Control:**
- Version: 1.0
- Status: Active
- Next Review: 2026-01-02
- Approved By: [Compliance Officer]
- Date: 2025-10-02
