-- Comprehensive Audit Log for GDPR Compliance
-- Tracks all data access, modifications, and consent changes

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    user_id TEXT NOT NULL DEFAULT 'default_user',
    action_type TEXT NOT NULL, -- 'consent_granted', 'consent_revoked', 'data_exported', 'data_deleted', 'data_accessed', 'data_modified'
    entity_type TEXT NOT NULL, -- 'document', 'chat_message', 'consent', 'user_setting'
    entity_id TEXT,            -- ID of the affected entity
    details TEXT,              -- JSON with additional context
    ip_address TEXT,           -- Optional: for security audit
    user_agent TEXT,           -- Optional: for security audit
    success BOOLEAN NOT NULL DEFAULT 1,
    error_message TEXT
);

-- Indexes for audit queries
CREATE INDEX IF NOT EXISTS idx_audit_timestamp
    ON audit_log(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_audit_user_action
    ON audit_log(user_id, action_type, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_audit_entity
    ON audit_log(entity_type, entity_id, timestamp DESC);
