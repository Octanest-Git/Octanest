-- logical: 0025_repo_activity_branch_rename — allow branch_rename push_type

PRAGMA foreign_keys = OFF;
CREATE TABLE repository_activity_new (
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
  commits_count   INTEGER NOT NULL DEFAULT 0,
  commit_message  TEXT NULL,
  pr_number       INTEGER NULL,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);
INSERT INTO repository_activity_new
  SELECT id, repository_id, actor_id, push_type, ref_name, before_oid, after_oid,
         commits_count, commit_message, pr_number, created_at
  FROM repository_activity;
DROP TABLE repository_activity;
ALTER TABLE repository_activity_new RENAME TO repository_activity;
CREATE INDEX IF NOT EXISTS idx_repository_activity_repo_created
  ON repository_activity(repository_id, created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_repository_activity_repo_type_created
  ON repository_activity(repository_id, push_type, created_at DESC);
PRAGMA foreign_keys = ON;
