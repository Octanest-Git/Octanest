-- logical: 0023_repo_about — homepage, watches, topics, fork_count (issue #23)

ALTER TABLE repositories ADD COLUMN homepage VARCHAR(2048) NOT NULL DEFAULT '';
ALTER TABLE repositories ADD COLUMN watch_count BIGINT NOT NULL DEFAULT 0;
ALTER TABLE repositories ADD COLUMN fork_count BIGINT NOT NULL DEFAULT 0;

-- Backfill fork_count via derived table (MySQL forbids same-table subquery in UPDATE).
UPDATE repositories r
INNER JOIN (
  SELECT fork_network_id, COUNT(*) AS cnt
  FROM repositories
  WHERE forked_from_repo_id IS NOT NULL
    AND deleted_at IS NULL
    AND fork_network_id IS NOT NULL
  GROUP BY fork_network_id
) c ON r.fork_network_id = c.fork_network_id
SET r.fork_count = c.cnt;

CREATE TABLE IF NOT EXISTS repository_watches (
  user_id       CHAR(36) NOT NULL,
  repository_id CHAR(36) NOT NULL,
  created_at    DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  PRIMARY KEY (user_id, repository_id),
  CONSTRAINT fk_repository_watches_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_repository_watches_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
);

CREATE INDEX idx_repository_watches_repo ON repository_watches(repository_id);
CREATE INDEX idx_repository_watches_user_created ON repository_watches(user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS topics (
  id         VARCHAR(64) PRIMARY KEY,
  name       VARCHAR(64) NOT NULL,
  created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  UNIQUE KEY uq_topics_name (name)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS repository_topics (
  repository_id CHAR(36) NOT NULL,
  topic_id      VARCHAR(64) NOT NULL,
  PRIMARY KEY (repository_id, topic_id),
  CONSTRAINT fk_repository_topics_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_repository_topics_topic FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_repository_topics_repo ON repository_topics(repository_id);
