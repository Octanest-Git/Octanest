-- logical: 0027_signing_keys — SSH usage flags + GPG public keys for commit signing

ALTER TABLE ssh_public_keys ADD COLUMN can_authenticate INTEGER NOT NULL DEFAULT 1;
ALTER TABLE ssh_public_keys ADD COLUMN can_sign INTEGER NOT NULL DEFAULT 1;

CREATE TABLE IF NOT EXISTS gpg_public_keys (
  id                   TEXT    PRIMARY KEY,
  user_id              TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  title                TEXT    NOT NULL,
  armored_public_key   TEXT    NOT NULL,
  fingerprint          TEXT    NOT NULL UNIQUE,
  key_id               TEXT    NOT NULL,
  uid_emails           TEXT    NOT NULL DEFAULT '[]',
  created_at           TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE INDEX IF NOT EXISTS idx_gpg_keys_user_id ON gpg_public_keys(user_id);
