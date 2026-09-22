-- logical: 0028_user_emails — multi-address account emails + per-target verify tokens
-- MySQL: no IF NOT EXISTS on ADD COLUMN — idempotent via migrate runner / fresh DBs.

CREATE TABLE IF NOT EXISTS user_emails (
  id          CHAR(36)      PRIMARY KEY,
  user_id     CHAR(36)      NOT NULL,
  email       VARCHAR(320)  NOT NULL,
  is_primary  TINYINT(1)    NOT NULL DEFAULT 0,
  verified_at TIMESTAMP     NULL,
  created_at  TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_user_emails_email (email),
  CONSTRAINT fk_user_emails_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_user_emails_user_id ON user_emails(user_id);

INSERT INTO user_emails (id, user_id, email, is_primary, verified_at, created_at)
SELECT
  UUID(),
  u.id,
  u.email,
  1,
  u.email_verified_at,
  u.created_at
FROM users u
WHERE NOT EXISTS (
  SELECT 1 FROM user_emails ue WHERE ue.user_id = u.id AND ue.is_primary = 1
);

ALTER TABLE auth_email_tokens
  ADD COLUMN target_email VARCHAR(320) NOT NULL DEFAULT '';

ALTER TABLE auth_email_tokens
  DROP INDEX user_id;

ALTER TABLE auth_email_tokens
  ADD UNIQUE KEY uk_auth_email_tokens_user_purpose_target (user_id, purpose, target_email);
