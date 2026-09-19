-- logical: 0024_repo_activity — durable push / branch / merge activity feed

CREATE TABLE IF NOT EXISTS repository_activity (
  id              TEXT PRIMARY KEY,
  repository_id   TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  actor_id        TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  push_type       TEXT NOT NULL
    CHECK (push_type IN (
      'push', 'force_push', 'pr_merge', 'branch_creation', 'branch_deletion', 'branch_rename'
    )),
  ref_name        TEXT NOT NULL,
  before_oid      TEXT NOT NULL DEFAULT '',
  after_oid       TEXT NOT NULL DEFAULT '',
  commits_count   BIGINT NOT NULL DEFAULT 0,
  commit_message  TEXT NULL,
  pr_number       BIGINT NULL,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_repository_activity_repo_created
  ON repository_activity(repository_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_repository_activity_repo_type_created
  ON repository_activity(repository_id, push_type, created_at DESC);
