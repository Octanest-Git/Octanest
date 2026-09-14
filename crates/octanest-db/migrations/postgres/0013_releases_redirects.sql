-- logical: 0013_releases_redirects — releases, assets metadata, rename redirects (GIT-14..17 / D-REL-*)

CREATE TABLE IF NOT EXISTS releases (
  id          TEXT        PRIMARY KEY,
  repo_id     TEXT        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  tag_name    TEXT        NOT NULL,
  title       TEXT        NOT NULL DEFAULT '',
  body        TEXT        NOT NULL DEFAULT '',
  draft       BOOLEAN     NOT NULL DEFAULT FALSE,
  prerelease  BOOLEAN     NOT NULL DEFAULT FALSE,
  author_id   TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT releases_repo_tag_unique UNIQUE (repo_id, tag_name)
);

CREATE INDEX IF NOT EXISTS idx_releases_repo_id ON releases(repo_id);
CREATE INDEX IF NOT EXISTS idx_releases_repo_draft ON releases(repo_id, draft);

CREATE TABLE IF NOT EXISTS release_assets (
  id            TEXT        PRIMARY KEY,
  release_id    TEXT        NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
  filename      TEXT        NOT NULL,
  content_type  TEXT        NOT NULL DEFAULT 'application/octet-stream',
  byte_size     BIGINT      NOT NULL DEFAULT 0,
  uploader_id   TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT release_assets_release_filename_unique UNIQUE (release_id, filename)
);

CREATE INDEX IF NOT EXISTS idx_release_assets_release_id ON release_assets(release_id);

CREATE TABLE IF NOT EXISTS repository_redirects (
  id              TEXT        PRIMARY KEY,
  old_owner_slug  TEXT        NOT NULL,
  old_name        TEXT        NOT NULL,
  repo_id         TEXT        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  expires_at      TIMESTAMPTZ NOT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT repository_redirects_slug_name_unique UNIQUE (old_owner_slug, old_name)
);

CREATE INDEX IF NOT EXISTS idx_repository_redirects_repo_id ON repository_redirects(repo_id);
CREATE INDEX IF NOT EXISTS idx_repository_redirects_expires ON repository_redirects(expires_at);
