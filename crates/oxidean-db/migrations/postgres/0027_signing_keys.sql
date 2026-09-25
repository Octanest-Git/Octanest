-- logical: 0027_signing_keys — SSH usage flags + GPG public keys for commit signing

ALTER TABLE ssh_public_keys
  ADD COLUMN can_authenticate BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE ssh_public_keys
  ADD COLUMN can_sign BOOLEAN NOT NULL DEFAULT true;

CREATE TABLE IF NOT EXISTS gpg_public_keys (
  id                   TEXT        PRIMARY KEY,
  user_id              TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  title                TEXT        NOT NULL,
  armored_public_key   TEXT        NOT NULL,
  fingerprint          TEXT        NOT NULL UNIQUE,
  key_id               TEXT        NOT NULL,
  uid_emails           TEXT        NOT NULL DEFAULT '[]',
  created_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_gpg_keys_user_id ON gpg_public_keys(user_id);
