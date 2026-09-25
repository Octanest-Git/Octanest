-- logical: 0027_signing_keys — SSH usage flags + GPG public keys for commit signing
-- MySQL: no IF NOT EXISTS on ADD COLUMN — idempotent via migrate runner / fresh DBs.

ALTER TABLE ssh_public_keys
  ADD COLUMN can_authenticate TINYINT(1) NOT NULL DEFAULT 1;
ALTER TABLE ssh_public_keys
  ADD COLUMN can_sign TINYINT(1) NOT NULL DEFAULT 1;

CREATE TABLE IF NOT EXISTS gpg_public_keys (
  id                   CHAR(36)      PRIMARY KEY,
  user_id              CHAR(36)      NOT NULL,
  title                VARCHAR(200)  NOT NULL,
  armored_public_key   TEXT          NOT NULL,
  fingerprint          VARCHAR(128)  NOT NULL,
  key_id               VARCHAR(64)   NOT NULL,
  uid_emails           TEXT          NOT NULL,
  created_at           TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_gpg_keys_fingerprint (fingerprint),
  CONSTRAINT fk_gpg_keys_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_gpg_keys_user_id ON gpg_public_keys(user_id);
