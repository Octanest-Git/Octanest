-- logical: 0028_user_emails — multi-address account emails + per-target verify tokens

CREATE TABLE IF NOT EXISTS user_emails (
  id          TEXT    PRIMARY KEY,
  user_id     TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  email       TEXT    NOT NULL UNIQUE,
  is_primary  INTEGER NOT NULL DEFAULT 0,
  verified_at TEXT    NULL,
  created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE INDEX IF NOT EXISTS idx_user_emails_user_id ON user_emails(user_id);

INSERT INTO user_emails (id, user_id, email, is_primary, verified_at, created_at)
SELECT
  id || '-primary-email',
  id,
  email,
  1,
  email_verified_at,
  created_at
FROM users
WHERE NOT EXISTS (
  SELECT 1 FROM user_emails ue WHERE ue.user_id = users.id AND ue.is_primary = 1
);

-- Recreate auth_email_tokens with target_email in the unique key.
CREATE TABLE auth_email_tokens_new (
  id            TEXT    PRIMARY KEY,
  user_id       TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  purpose       TEXT    NOT NULL,
  target_email  TEXT    NOT NULL DEFAULT '',
  token_hash    TEXT    NOT NULL,
  otp_hash      TEXT    NOT NULL,
  expires_at    TEXT    NOT NULL,
  attempt_count INTEGER NOT NULL DEFAULT 0,
  issue_count   INTEGER NOT NULL DEFAULT 1,
  created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  UNIQUE (user_id, purpose, target_email)
);

INSERT INTO auth_email_tokens_new (
  id, user_id, purpose, target_email, token_hash, otp_hash, expires_at,
  attempt_count, issue_count, created_at
)
SELECT
  id, user_id, purpose, '', token_hash, otp_hash, expires_at,
  attempt_count, issue_count, created_at
FROM auth_email_tokens;

DROP TABLE auth_email_tokens;
ALTER TABLE auth_email_tokens_new RENAME TO auth_email_tokens;

CREATE INDEX IF NOT EXISTS idx_auth_email_tokens_token_hash ON auth_email_tokens(token_hash);
