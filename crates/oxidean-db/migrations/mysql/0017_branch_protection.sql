-- logical: 0017_branch_protection — classic rules + commit statuses (Phase 13)

CREATE TABLE IF NOT EXISTS branch_protection_rules (
  id                              CHAR(36)      PRIMARY KEY,
  repo_id                         CHAR(36)      NOT NULL,
  pattern                         VARCHAR(255)  NOT NULL,
  require_reviews                 TINYINT(1)    NOT NULL DEFAULT 0,
  required_approving_review_count INT           NOT NULL DEFAULT 1,
  dismiss_stale_reviews           TINYINT(1)    NOT NULL DEFAULT 0,
  require_conversation_resolution TINYINT(1)    NOT NULL DEFAULT 0,
  require_last_push_approval      TINYINT(1)    NOT NULL DEFAULT 0,
  required_status_contexts        TEXT          NOT NULL DEFAULT ('[]'),
  strict_status_checks            TINYINT(1)    NOT NULL DEFAULT 0,
  allow_force_pushes              TINYINT(1)    NOT NULL DEFAULT 0,
  allow_deletions                 TINYINT(1)    NOT NULL DEFAULT 0,
  enforce_admins                  TINYINT(1)    NOT NULL DEFAULT 0,
  required_linear_history         TINYINT(1)    NOT NULL DEFAULT 0,
  lock_branch                     TINYINT(1)    NOT NULL DEFAULT 0,
  created_at                      TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at                      TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_branch_protection_rules_repo
    FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_branch_protection_rules_repo ON branch_protection_rules(repo_id);

CREATE TABLE IF NOT EXISTS commit_statuses (
  id           CHAR(36)      PRIMARY KEY,
  repo_id      CHAR(36)      NOT NULL,
  sha          VARCHAR(64)   NOT NULL,
  context      VARCHAR(255)  NOT NULL,
  state        VARCHAR(16)   NOT NULL,
  description  TEXT          NOT NULL,
  target_url   VARCHAR(2048) NULL,
  creator_id   CHAR(36)      NULL,
  created_at   TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at   TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_commit_statuses_repo_sha_ctx (repo_id, sha, context),
  CONSTRAINT fk_commit_statuses_repo
    FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_commit_statuses_creator
    FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE SET NULL,
  CONSTRAINT commit_statuses_state_check
    CHECK (state IN ('pending', 'success', 'failure', 'error'))
) ENGINE=InnoDB;

CREATE INDEX idx_commit_statuses_repo_sha ON commit_statuses(repo_id, sha);
