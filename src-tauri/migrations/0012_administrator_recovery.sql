CREATE TABLE administrator_recovery_codes (
  user_id TEXT PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
  code_hash TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE administrator_recovery_attempts (
  username_hash TEXT PRIMARY KEY,
  failed_attempts INTEGER NOT NULL DEFAULT 0 CHECK (failed_attempts >= 0),
  locked_until INTEGER NOT NULL DEFAULT 0
);
