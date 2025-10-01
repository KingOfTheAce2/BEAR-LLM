-- Consent Version Management
-- Tracks different versions of consent agreements for compliance

CREATE TABLE IF NOT EXISTS consent_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    consent_type TEXT NOT NULL,
    version INTEGER NOT NULL,
    consent_text TEXT NOT NULL,
    effective_date DATETIME NOT NULL,
    deprecated_date DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(consent_type, version)
);

-- Index for active consent version lookups
CREATE INDEX IF NOT EXISTS idx_consent_versions_active
    ON consent_versions(consent_type, effective_date, deprecated_date);

-- Insert default consent versions
INSERT OR IGNORE INTO consent_versions (consent_type, version, consent_text, effective_date) VALUES
('pii_detection', 1, 'I consent to automated PII detection and redaction of my documents and chat messages for privacy protection.', datetime('now')),
('chat_storage', 1, 'I consent to storage of my chat conversations for providing AI assistance. I understand I can export or delete this data at any time.', datetime('now')),
('document_processing', 1, 'I consent to processing and analysis of uploaded documents for knowledge base features. All documents are stored locally.', datetime('now')),
('analytics', 1, 'I consent to collection of anonymized usage analytics to improve the application.', datetime('now'));
