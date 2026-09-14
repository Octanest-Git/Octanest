-- logical: 0003_email_tokens — verify/reset magic+OTP hashes (hash-at-rest only)
CREATE TABLE IF NOT EXISTS auth_email_tokens (
  id            TEXT        PRIMARY KEY,
  user_id       TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  purpose       TEXT        NOT NULL,
  token_hash    CHAR(64)    NOT NULL,
  otp_hash      CHAR(64)    NOT NULL,
  expires_at    TIMESTAMPTZ NOT NULL,
  attempt_count INTEGER     NOT NULL DEFAULT 0,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, purpose)
);

CREATE INDEX IF NOT EXISTS idx_auth_email_tokens_token_hash ON auth_email_tokens(token_hash);
