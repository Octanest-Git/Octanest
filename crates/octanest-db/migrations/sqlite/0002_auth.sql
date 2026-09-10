-- logical: 0002_auth — users/sessions/identities/settings
CREATE TABLE IF NOT EXISTS users (
  id                TEXT    PRIMARY KEY,
  email             TEXT    NOT NULL UNIQUE,
  username          TEXT    NOT NULL UNIQUE,
  password_hash     TEXT    NULL,
  display_name      TEXT    NOT NULL,
  bio               TEXT    NOT NULL DEFAULT '',
  avatar_path       TEXT    NULL,
  is_admin          INTEGER NOT NULL DEFAULT 0,
  email_verified_at TEXT    NULL,
  created_at        TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  updated_at        TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE TABLE IF NOT EXISTS sessions (
  id           TEXT    PRIMARY KEY,
  user_id      TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash   TEXT    NOT NULL UNIQUE,
  expires_at   TEXT    NOT NULL,
  remember_me  INTEGER NOT NULL DEFAULT 0,
  created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  last_seen_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);

CREATE TABLE IF NOT EXISTS auth_identities (
  id               TEXT PRIMARY KEY,
  user_id          TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider         TEXT NOT NULL,
  provider_subject TEXT NOT NULL,
  provider_email   TEXT NULL,
  UNIQUE (provider, provider_subject)
);

CREATE TABLE IF NOT EXISTS instance_auth_settings (
  id               INTEGER PRIMARY KEY CHECK (id = 1),
  provider_mode    TEXT NOT NULL DEFAULT 'local',
  email_provider   TEXT NOT NULL DEFAULT 'log',
  from_address     TEXT NULL,
  oidc_issuer      TEXT NULL,
  oidc_client_id   TEXT NULL,
  workos_client_id TEXT NULL,
  updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

INSERT OR IGNORE INTO instance_auth_settings (id, provider_mode, email_provider)
VALUES (1, 'local', 'log');
