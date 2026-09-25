-- logical: 0007_repositories — repositories + user default_branch + instance default_visibility
CREATE TABLE IF NOT EXISTS repositories (
  id             TEXT        PRIMARY KEY,
  owner_id       TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name           VARCHAR(100) NOT NULL,
  visibility     TEXT        NOT NULL DEFAULT 'public',
  description    TEXT        NOT NULL DEFAULT '',
  default_branch TEXT        NOT NULL DEFAULT 'main',
  deleted_at     TIMESTAMPTZ NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Unique among non-deleted (owner_id, lower(name)) — GIT-01 / T-07-05a
CREATE UNIQUE INDEX IF NOT EXISTS idx_repositories_owner_name_active
  ON repositories (owner_id, lower(name))
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_repositories_owner_id ON repositories(owner_id);

ALTER TABLE users ADD COLUMN default_branch TEXT NOT NULL DEFAULT 'main';
ALTER TABLE instance_auth_settings ADD COLUMN default_visibility TEXT NOT NULL DEFAULT 'public';
