-- logical: 0028_user_emails — multi-address account emails + per-target verify tokens

CREATE TABLE IF NOT EXISTS user_emails (
  id          TEXT        PRIMARY KEY,
  user_id     TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  email       TEXT        NOT NULL UNIQUE,
  is_primary  BOOLEAN     NOT NULL DEFAULT false,
  verified_at TIMESTAMPTZ NULL,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_user_emails_user_id ON user_emails(user_id);

INSERT INTO user_emails (id, user_id, email, is_primary, verified_at, created_at)
SELECT
  u.id || '-primary-email',
  u.id,
  u.email,
  true,
  u.email_verified_at,
  u.created_at
FROM users u
WHERE NOT EXISTS (
  SELECT 1 FROM user_emails ue WHERE ue.user_id = u.id AND ue.is_primary = true
);

ALTER TABLE auth_email_tokens
  ADD COLUMN IF NOT EXISTS target_email TEXT NOT NULL DEFAULT '';

ALTER TABLE auth_email_tokens
  DROP CONSTRAINT IF EXISTS auth_email_tokens_user_id_purpose_key;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'auth_email_tokens_user_purpose_target_unique'
  ) THEN
    ALTER TABLE auth_email_tokens
      ADD CONSTRAINT auth_email_tokens_user_purpose_target_unique
      UNIQUE (user_id, purpose, target_email);
  END IF;
END $$;
