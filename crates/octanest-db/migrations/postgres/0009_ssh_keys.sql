-- logical: 0009_ssh_keys — registered SSH public keys (fingerprint UNIQUE; D-SSH-05)
CREATE TABLE IF NOT EXISTS ssh_public_keys (
  id             TEXT        PRIMARY KEY,
  user_id        TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  title          TEXT        NOT NULL,
  public_key     TEXT        NOT NULL,
  fingerprint    TEXT        NOT NULL UNIQUE,
  key_type       TEXT        NOT NULL,
  last_used_at   TIMESTAMPTZ NULL,
  last_used_ip   TEXT        NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ssh_keys_user_id ON ssh_public_keys(user_id);
