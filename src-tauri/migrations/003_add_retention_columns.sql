-- Data Retention Management
-- Adds retention_until columns to support automated data deletion

-- Add retention to documents table
ALTER TABLE documents ADD COLUMN retention_until DATETIME;

-- Add retention to chat_sessions table
ALTER TABLE chat_sessions ADD COLUMN retention_until DATETIME;

-- Add retention to chat_messages table
ALTER TABLE chat_messages ADD COLUMN retention_until DATETIME;

-- Add retention to query_history table
ALTER TABLE query_history ADD COLUMN retention_until DATETIME;

-- Index for efficient retention cleanup queries
CREATE INDEX IF NOT EXISTS idx_documents_retention
    ON documents(retention_until) WHERE retention_until IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_chat_sessions_retention
    ON chat_sessions(retention_until) WHERE retention_until IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_chat_messages_retention
    ON chat_messages(retention_until) WHERE retention_until IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_query_history_retention
    ON query_history(retention_until) WHERE retention_until IS NOT NULL;
