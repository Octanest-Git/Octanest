-- logical: 0002_auth — users/sessions/identities/settings
CREATE TABLE IF NOT EXISTS users (
  id                TEXT        PRIMARY KEY,
  email             TEXT        NOT NULL UNIQUE,
  username          VARCHAR(39) NOT NULL UNIQUE,
  password_hash     TEXT        NULL,
  display_name      TEXT        NOT NULL,
  bio               TEXT        NOT NULL DEFAULT '',
  avatar_path       TEXT        NULL,
  is_admin          BOOLEAN     NOT NULL DEFAULT FALSE,
  email_verified_at TIMESTAMPTZ NULL,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS sessions (
  id           TEXT        PRIMARY KEY,
  user_id      TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash   CHAR(64)    NOT NULL UNIQUE,
  expires_at   TIMESTAMPTZ NOT NULL,
  remember_me  BOOLEAN     NOT NULL DEFAULT FALSE,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
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
  provider_mode    TEXT        NOT NULL DEFAULT 'local',
  email_provider   TEXT        NOT NULL DEFAULT 'log',
  from_address     TEXT        NULL,
  oidc_issuer      TEXT        NULL,
  oidc_client_id   TEXT        NULL,
  workos_client_id TEXT        NULL,
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO instance_auth_settings (id, provider_mode, email_provider)
VALUES (1, 'local', 'log')
ON CONFLICT (id) DO NOTHING;
