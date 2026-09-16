-- logical: 0020_social — stars + fork_network_id (Phase 21; forked_from_repo_id from 0016)
-- D-SOC-02, D-SOC-14. Dialect SQL only.

CREATE TABLE IF NOT EXISTS repository_stars (
  user_id       CHAR(36) NOT NULL,
  repository_id CHAR(36) NOT NULL,
  created_at    DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  PRIMARY KEY (user_id, repository_id),
  CONSTRAINT fk_repository_stars_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_repository_stars_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
);

CREATE INDEX idx_repository_stars_repo ON repository_stars(repository_id);
CREATE INDEX idx_repository_stars_user_created ON repository_stars(user_id, created_at DESC);

ALTER TABLE repositories ADD COLUMN star_count BIGINT NOT NULL DEFAULT 0;
ALTER TABLE repositories ADD COLUMN fork_network_id CHAR(36) NULL;

UPDATE repositories SET fork_network_id = id WHERE fork_network_id IS NULL;

ALTER TABLE repositories
  ADD CONSTRAINT fk_repositories_fork_network
  FOREIGN KEY (fork_network_id) REFERENCES repositories(id) ON DELETE SET NULL;

CREATE INDEX idx_repositories_fork_network ON repositories(fork_network_id);
CREATE INDEX idx_repositories_explore ON repositories(visibility, star_count, updated_at);
