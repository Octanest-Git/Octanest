-- logical: 0022_templates — instance template packs + user template repos (issue #18)

ALTER TABLE repositories ADD COLUMN is_template BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE repositories ADD COLUMN created_from_template_repo_id TEXT NULL REFERENCES repositories(id) ON DELETE SET NULL;

CREATE TABLE IF NOT EXISTS instance_template_packs (
  id                   TEXT PRIMARY KEY,
  slug                 TEXT NOT NULL UNIQUE,
  label                TEXT NOT NULL,
  "group"              TEXT NOT NULL DEFAULT 'Custom',
  description          TEXT NOT NULL DEFAULT '',
  default_gitignore    TEXT NULL,
  enabled              BOOLEAN NOT NULL DEFAULT true,
  byte_size            BIGINT NOT NULL,
  content_digest       TEXT NOT NULL,
  uploaded_by_user_id  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_instance_template_packs_enabled ON instance_template_packs(enabled);
CREATE INDEX IF NOT EXISTS idx_repositories_is_template ON repositories(is_template) WHERE is_template = true;
