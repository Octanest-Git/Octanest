-- logical: 0023_repo_about — homepage, watches, topics, fork_count (issue #23)

ALTER TABLE repositories ADD COLUMN homepage TEXT NOT NULL DEFAULT '';
ALTER TABLE repositories ADD COLUMN watch_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE repositories ADD COLUMN fork_count INTEGER NOT NULL DEFAULT 0;

-- Backfill via nested SELECT so SQLite accepts same-table aggregate.
UPDATE repositories
SET fork_count = COALESCE((
  SELECT cnt FROM (
    SELECT fork_network_id AS nid, COUNT(*) AS cnt
    FROM repositories
    WHERE forked_from_repo_id IS NOT NULL
      AND deleted_at IS NULL
      AND fork_network_id IS NOT NULL
    GROUP BY fork_network_id
  )
  WHERE nid = repositories.fork_network_id
), 0)
WHERE fork_network_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS repository_watches (
  user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  PRIMARY KEY (user_id, repository_id)
);

CREATE INDEX IF NOT EXISTS idx_repository_watches_repo
  ON repository_watches(repository_id);
CREATE INDEX IF NOT EXISTS idx_repository_watches_user_created
  ON repository_watches(user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS topics (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE TABLE IF NOT EXISTS repository_topics (
  repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  topic_id      TEXT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
  PRIMARY KEY (repository_id, topic_id)
);

CREATE INDEX IF NOT EXISTS idx_repository_topics_repo
  ON repository_topics(repository_id);
