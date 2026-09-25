-- logical: 0003_email_tokens — verify/reset magic+OTP hashes (hash-at-rest only)
CREATE TABLE IF NOT EXISTS auth_email_tokens (
  id            CHAR(36)  PRIMARY KEY,
  user_id       CHAR(36)  NOT NULL,
  purpose       VARCHAR(16) NOT NULL,
  token_hash    CHAR(64)  NOT NULL,
  otp_hash      CHAR(64)  NOT NULL,
  expires_at    TIMESTAMP NOT NULL,
  attempt_count INT       NOT NULL DEFAULT 0,
  created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (user_id, purpose),
  CONSTRAINT fk_auth_email_tokens_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_auth_email_tokens_token_hash ON auth_email_tokens(token_hash);
