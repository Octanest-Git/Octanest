-- logical: 0024_repo_activity — durable push / branch / merge activity feed

CREATE TABLE IF NOT EXISTS repository_activity (
  id              CHAR(36) NOT NULL,
  repository_id   CHAR(36) NOT NULL,
  actor_id        CHAR(36) NOT NULL,
  push_type       VARCHAR(32) NOT NULL,
  ref_name        VARCHAR(512) NOT NULL,
  before_oid      VARCHAR(64) NOT NULL DEFAULT '',
  after_oid       VARCHAR(64) NOT NULL DEFAULT '',
  commits_count   BIGINT NOT NULL DEFAULT 0,
  commit_message  TEXT NULL,
  pr_number       BIGINT NULL,
  created_at      DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  PRIMARY KEY (id),
  CONSTRAINT fk_repository_activity_repo
    FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_repository_activity_actor
    FOREIGN KEY (actor_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT chk_repository_activity_push_type
    CHECK (push_type IN (
      'push', 'force_push', 'pr_merge', 'branch_creation', 'branch_deletion', 'branch_rename'
    ))
) ENGINE=InnoDB;

CREATE INDEX idx_repository_activity_repo_created
  ON repository_activity(repository_id, created_at DESC, id DESC);

CREATE INDEX idx_repository_activity_repo_type_created
  ON repository_activity(repository_id, push_type, created_at DESC);
