-- User Consent Table - GDPR Compliance
-- Tracks granular user consent for various data processing activities

CREATE TABLE IF NOT EXISTS user_consent (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL DEFAULT 'default_user', -- Multi-user support ready
    consent_type TEXT NOT NULL,  -- 'pii_detection', 'chat_storage', 'document_processing', 'analytics'
    granted BOOLEAN NOT NULL DEFAULT 0,
    granted_at DATETIME,
    revoked_at DATETIME,
    version INTEGER NOT NULL DEFAULT 1,
    consent_text TEXT NOT NULL,  -- The exact text user consented to
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(user_id, consent_type, version)
);

-- Index for fast consent lookups
CREATE INDEX IF NOT EXISTS idx_user_consent_lookup
    ON user_consent(user_id, consent_type, granted);

-- Index for consent audit queries
CREATE INDEX IF NOT EXISTS idx_user_consent_audit
    ON user_consent(user_id, granted_at, revoked_at);
