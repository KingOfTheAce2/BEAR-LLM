-- CRITICAL SECURITY FIX: Add user_id columns for data isolation
-- This migration fixes the severe privacy violation where all users could see each other's data
--
-- Before this migration: fetch_user_data() returned ALL users' data regardless of user_id
-- After this migration: Each user can only see their own data
--
-- GDPR Compliance: Article 15 (Right of Access) - users must only see their own data

-- Add user_id column to chat_sessions
ALTER TABLE chat_sessions ADD COLUMN user_id TEXT NOT NULL DEFAULT 'default_user';

-- Add user_id column to chat_messages
ALTER TABLE chat_messages ADD COLUMN user_id TEXT NOT NULL DEFAULT 'default_user';

-- Add user_id column to documents
ALTER TABLE documents ADD COLUMN user_id TEXT NOT NULL DEFAULT 'default_user';

-- Add user_id column to pii_detections
ALTER TABLE pii_detections ADD COLUMN user_id TEXT NOT NULL DEFAULT 'default_user';

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_chat_sessions_user_id ON chat_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_user_id ON chat_messages(user_id);
CREATE INDEX IF NOT EXISTS idx_documents_user_id ON documents(user_id);
CREATE INDEX IF NOT EXISTS idx_pii_detections_user_id ON pii_detections(user_id);

-- Note: Foreign key constraints would require a users table to exist
-- If you have a users table, uncomment and add:
-- ALTER TABLE chat_sessions ADD CONSTRAINT fk_chat_sessions_user
--     FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
-- ALTER TABLE chat_messages ADD CONSTRAINT fk_chat_messages_user
--     FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
-- ALTER TABLE documents ADD CONSTRAINT fk_documents_user
--     FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
