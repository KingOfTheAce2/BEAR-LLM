-- GDPR Article 30 - Records of Processing Activities
-- Maintains comprehensive log of all data processing operations
-- Required for GDPR compliance and regulatory audits

CREATE TABLE IF NOT EXISTS processing_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    processing_purpose TEXT NOT NULL,      -- Purpose of processing (e.g., "Document Analysis", "Chat Storage")
    data_categories TEXT NOT NULL,         -- Categories of data processed (JSON array)
    legal_basis TEXT NOT NULL,             -- Legal basis: 'consent', 'contract', 'legal_obligation', 'legitimate_interest'
    retention_period INTEGER NOT NULL,     -- Retention period in days
    recipients TEXT,                       -- Recipients of data (JSON array, e.g., ["AI Provider", "Cloud Storage"])
    controller_info TEXT NOT NULL,         -- Data controller information (JSON object)
    data_subject_categories TEXT,          -- Categories of data subjects (e.g., "legal professionals", "clients")
    international_transfers TEXT,          -- Details of international transfers (JSON object)
    security_measures TEXT,                -- Technical and organizational security measures (JSON array)
    user_id TEXT NOT NULL DEFAULT 'default_user',
    entity_type TEXT NOT NULL,             -- Type of entity: 'document', 'chat', 'query'
    entity_id TEXT,                        -- ID of processed entity
    metadata TEXT                          -- Additional metadata (JSON)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_processing_timestamp
    ON processing_records(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_processing_user
    ON processing_records(user_id, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_processing_purpose
    ON processing_records(processing_purpose, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_processing_legal_basis
    ON processing_records(legal_basis, timestamp DESC);
