-- logical: 0007_repositories — repositories + user default_branch + instance default_visibility
-- MySQL: TEXT cannot carry DEFAULT — use VARCHAR. Partial unique via generated column (NULLs allowed in UNIQUE).
CREATE TABLE IF NOT EXISTS repositories (
  id             CHAR(36)     PRIMARY KEY,
  owner_id       CHAR(36)     NOT NULL,
  name           VARCHAR(100) NOT NULL,
  visibility     VARCHAR(16)  NOT NULL DEFAULT 'public',
  description    VARCHAR(4000) NOT NULL DEFAULT '',
  default_branch VARCHAR(255) NOT NULL DEFAULT 'main',
  deleted_at     TIMESTAMP    NULL,
  created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  -- NULL when soft-deleted so UNIQUE permits reuse of the name among deleted rows
  active_name    VARCHAR(100) GENERATED ALWAYS AS (
    CASE WHEN deleted_at IS NULL THEN LOWER(name) ELSE NULL END
  ) STORED,
  CONSTRAINT fk_repositories_owner FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE,
  UNIQUE KEY uk_repositories_owner_active_name (owner_id, active_name)
) ENGINE=InnoDB;

CREATE INDEX idx_repositories_owner_id ON repositories(owner_id);

ALTER TABLE users ADD COLUMN default_branch VARCHAR(255) NOT NULL DEFAULT 'main';
ALTER TABLE instance_auth_settings ADD COLUMN default_visibility VARCHAR(16) NOT NULL DEFAULT 'public';
