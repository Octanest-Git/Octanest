-- logical: 0026_repo_mirrors — two-way repository mirroring (GIT-V2-01)

CREATE TABLE IF NOT EXISTS repository_mirrors (
  id                         CHAR(36) PRIMARY KEY,
  repository_id              CHAR(36) NOT NULL,
  remote_url                 TEXT NOT NULL,
  auth_kind                  VARCHAR(16) NOT NULL,
  username                   VARCHAR(255) NOT NULL DEFAULT '',
  secret_ciphertext          TEXT NOT NULL,
  ssh_public_key             TEXT NOT NULL,
  known_hosts                TEXT NOT NULL,
  webhook_secret_ciphertext  TEXT NOT NULL,
  poll_interval_secs         INT NOT NULL DEFAULT 60,
  enabled                    TINYINT(1) NOT NULL DEFAULT 1,
  last_synced_at             TIMESTAMP NULL,
  last_status                VARCHAR(16) NOT NULL DEFAULT 'never',
  last_error                 TEXT NOT NULL,
  created_at                 TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                 TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uq_repository_mirrors_repo (repository_id),
  CONSTRAINT fk_repository_mirrors_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_repository_mirrors_enabled ON repository_mirrors(enabled);

CREATE TABLE IF NOT EXISTS repository_mirror_ref_results (
  id              CHAR(36) PRIMARY KEY,
  mirror_id       CHAR(36) NOT NULL,
  refname         VARCHAR(512) NOT NULL,
  outcome         VARCHAR(16) NOT NULL,
  local_oid       VARCHAR(64) NOT NULL DEFAULT '',
  remote_oid      VARCHAR(64) NOT NULL DEFAULT '',
  detail          TEXT NOT NULL,
  updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uq_mirror_ref (mirror_id, refname),
  CONSTRAINT fk_mirror_ref_results_mirror FOREIGN KEY (mirror_id) REFERENCES repository_mirrors(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_repository_mirror_ref_results_mirror ON repository_mirror_ref_results(mirror_id);
