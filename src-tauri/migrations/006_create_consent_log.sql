-- Granular Consent Log - Enhanced GDPR Compliance
-- Tracks detailed consent history with versioning and withdrawal mechanism
-- Supports: analytics, ai_processing, data_retention consent types

CREATE TABLE IF NOT EXISTS consent_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL DEFAULT 'default_user',
    consent_type TEXT NOT NULL,  -- 'analytics', 'ai_processing', 'data_retention', 'pii_detection', 'chat_storage', 'document_processing'
    version TEXT NOT NULL,       -- Version of consent policy (e.g., "1.0.0", "2024-01-15")
    granted BOOLEAN NOT NULL,    -- true = granted, false = withdrawn
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,             -- IP address when consent was given/withdrawn
    user_agent TEXT,             -- User agent for audit trail
    consent_text TEXT NOT NULL,  -- Full text of consent policy at time of acceptance
    withdrawal_reason TEXT,      -- Optional reason for withdrawal
    metadata TEXT                -- Additional metadata (JSON)
);

-- Indexes for fast consent lookups and audit trails
CREATE INDEX IF NOT EXISTS idx_consent_log_user
    ON consent_log(user_id, consent_type, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_consent_log_type
    ON consent_log(consent_type, granted, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_consent_log_version
    ON consent_log(consent_type, version, timestamp DESC);

-- Trigger to log consent changes to audit_log
CREATE TRIGGER IF NOT EXISTS consent_log_audit
AFTER INSERT ON consent_log
BEGIN
    INSERT INTO audit_log (user_id, action_type, entity_type, entity_id, details, success)
    VALUES (
        NEW.user_id,
        CASE WHEN NEW.granted = 1 THEN 'consent_granted' ELSE 'consent_revoked' END,
        'consent',
        CAST(NEW.id AS TEXT),
        json_object(
            'consent_type', NEW.consent_type,
            'version', NEW.version,
            'ip_address', NEW.ip_address
        ),
        1
    );
END;
