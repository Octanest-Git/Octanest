-- logical: 0014_releases_redirects — releases, assets metadata, rename redirects (GIT-14..17 / D-REL-*)

CREATE TABLE IF NOT EXISTS releases (
  id          CHAR(36)       PRIMARY KEY,
  repo_id     CHAR(36)       NOT NULL,
  tag_name    VARCHAR(255)   NOT NULL,
  title       VARCHAR(500)   NOT NULL DEFAULT '',
  body        TEXT           NOT NULL DEFAULT (''),
  draft       TINYINT(1)     NOT NULL DEFAULT 0,
  prerelease  TINYINT(1)     NOT NULL DEFAULT 0,
  author_id   CHAR(36)       NOT NULL,
  created_at  TIMESTAMP      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at  TIMESTAMP      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_releases_repo_tag (repo_id, tag_name),
  CONSTRAINT fk_releases_repo FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_releases_author FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_releases_repo_id ON releases(repo_id);
CREATE INDEX idx_releases_repo_draft ON releases(repo_id, draft);

CREATE TABLE IF NOT EXISTS release_assets (
  id            CHAR(36)       PRIMARY KEY,
  release_id    CHAR(36)       NOT NULL,
  filename      VARCHAR(512)   NOT NULL,
  content_type  VARCHAR(255)   NOT NULL DEFAULT 'application/octet-stream',
  byte_size     BIGINT         NOT NULL DEFAULT 0,
  uploader_id   CHAR(36)       NOT NULL,
  created_at    TIMESTAMP      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    TIMESTAMP      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_release_assets_release_filename (release_id, filename),
  CONSTRAINT fk_release_assets_release FOREIGN KEY (release_id) REFERENCES releases(id) ON DELETE CASCADE,
  CONSTRAINT fk_release_assets_uploader FOREIGN KEY (uploader_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_release_assets_release_id ON release_assets(release_id);

CREATE TABLE IF NOT EXISTS repository_redirects (
  id              CHAR(36)     PRIMARY KEY,
  old_owner_slug  VARCHAR(100) NOT NULL,
  old_name        VARCHAR(100) NOT NULL,
  repo_id         CHAR(36)     NOT NULL,
  expires_at      TIMESTAMP    NOT NULL,
  created_at      TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_repository_redirects_slug_name (old_owner_slug, old_name),
  CONSTRAINT fk_repository_redirects_repo FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_repository_redirects_repo_id ON repository_redirects(repo_id);
CREATE INDEX idx_repository_redirects_expires ON repository_redirects(expires_at);
