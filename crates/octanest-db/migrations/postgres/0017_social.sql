-- logical: 0017_social — stars + fork_network_id (Phase 21; forked_from_repo_id from 0016)
-- D-SOC-02, D-SOC-14. Dialect SQL only.

CREATE TABLE IF NOT EXISTS repository_stars (
  user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, repository_id)
);

CREATE INDEX IF NOT EXISTS idx_repository_stars_repo
  ON repository_stars(repository_id);
CREATE INDEX IF NOT EXISTS idx_repository_stars_user_created
  ON repository_stars(user_id, created_at DESC);

ALTER TABLE repositories ADD COLUMN star_count BIGINT NOT NULL DEFAULT 0;
ALTER TABLE repositories ADD COLUMN fork_network_id TEXT NULL REFERENCES repositories(id) ON DELETE SET NULL;

UPDATE repositories SET fork_network_id = id WHERE fork_network_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_repositories_fork_network
  ON repositories(fork_network_id);
CREATE INDEX IF NOT EXISTS idx_repositories_explore
  ON repositories(visibility, star_count DESC, updated_at DESC);
