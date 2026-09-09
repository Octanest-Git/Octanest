-- logical: 0002_auth — users/sessions/identities/settings
CREATE TABLE IF NOT EXISTS users (
  id                CHAR(36)     PRIMARY KEY,
  email             VARCHAR(320) NOT NULL UNIQUE,
  username          VARCHAR(39)  NOT NULL UNIQUE,
  password_hash     TEXT         NULL,
  display_name      TEXT         NOT NULL,
  bio               TEXT         NOT NULL DEFAULT '',
  avatar_path       TEXT         NULL,
  is_admin          BOOLEAN      NOT NULL DEFAULT FALSE,
  email_verified_at TIMESTAMP    NULL,
  created_at        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS sessions (
  id           CHAR(36)  PRIMARY KEY,
  user_id      CHAR(36)  NOT NULL,
  token_hash   CHAR(64)  NOT NULL UNIQUE,
  expires_at   TIMESTAMP NOT NULL,
  remember_me  BOOLEAN   NOT NULL DEFAULT FALSE,
  created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_seen_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_sessions_user_id ON sessions(user_id);

CREATE TABLE IF NOT EXISTS auth_identities (
  id               CHAR(36)     PRIMARY KEY,
  user_id          CHAR(36)     NOT NULL,
  provider         VARCHAR(32)  NOT NULL,
  provider_subject VARCHAR(255) NOT NULL,
  provider_email   VARCHAR(320) NULL,
  UNIQUE (provider, provider_subject),
  CONSTRAINT fk_auth_identities_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS instance_auth_settings (
  id               INT       PRIMARY KEY,
  provider_mode    TEXT      NOT NULL DEFAULT 'local',
  email_provider   TEXT      NOT NULL DEFAULT 'log',
  from_address     TEXT      NULL,
  oidc_issuer      TEXT      NULL,
  oidc_client_id   TEXT      NULL,
  workos_client_id TEXT      NULL,
  updated_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT chk_instance_auth_settings_singleton CHECK (id = 1)
) ENGINE=InnoDB;

INSERT IGNORE INTO instance_auth_settings (id, provider_mode, email_provider)
VALUES (1, 'local', 'log');
