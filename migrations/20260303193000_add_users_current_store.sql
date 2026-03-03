ALTER TABLE users
ADD COLUMN IF NOT EXISTS current_store_id VARCHAR(26);

CREATE INDEX IF NOT EXISTS users_current_store_idx ON users (current_store_id);
