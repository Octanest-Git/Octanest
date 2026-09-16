-- logical: 0021_actions — Actions runners/runs/jobs/secrets + repo gate (D-ACT-13, D-ACT-17)
-- commit_statuses already from 0017_branch_protection (Phase 13 / D-ACT-15 consumer).

ALTER TABLE repositories ADD COLUMN actions_enabled TINYINT(1) NOT NULL DEFAULT 1;

CREATE TABLE IF NOT EXISTS action_runners (
  id            CHAR(36) PRIMARY KEY,
  name          VARCHAR(255) NOT NULL,
  token_hash    VARCHAR(128) NOT NULL,
  labels_json   TEXT NOT NULL DEFAULT ('[]'),
  owner_type    VARCHAR(16) NULL,
  owner_id      CHAR(36) NULL,
  repository_id CHAR(36) NULL,
  ephemeral     TINYINT(1) NOT NULL DEFAULT 0,
  last_online   TIMESTAMP NULL,
  created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_action_runners_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_action_runners_repository_id ON action_runners(repository_id);
CREATE INDEX idx_action_runners_token_hash ON action_runners(token_hash);

CREATE TABLE IF NOT EXISTS action_runner_tokens (
  id            CHAR(36) PRIMARY KEY,
  token_hash    VARCHAR(128) NOT NULL,
  scope_type    VARCHAR(16) NOT NULL,
  scope_id      CHAR(36) NULL,
  active        TINYINT(1) NOT NULL DEFAULT 1,
  created_by    CHAR(36) NULL,
  created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at    TIMESTAMP NULL,
  UNIQUE KEY uq_action_runner_tokens_hash (token_hash),
  CONSTRAINT fk_action_runner_tokens_user FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL
) ENGINE=InnoDB;

CREATE INDEX idx_action_runner_tokens_active ON action_runner_tokens(active);

CREATE TABLE IF NOT EXISTS action_runs (
  id             CHAR(36) PRIMARY KEY,
  repository_id  CHAR(36) NOT NULL,
  workflow_path  VARCHAR(512) NOT NULL,
  workflow_name  VARCHAR(255) NOT NULL DEFAULT '',
  event          VARCHAR(64) NOT NULL,
  head_sha       CHAR(40) NOT NULL,
  head_ref       VARCHAR(255) NOT NULL DEFAULT '',
  status         VARCHAR(32) NOT NULL DEFAULT 'queued',
  title          VARCHAR(512) NOT NULL DEFAULT '',
  triggered_by   CHAR(36) NULL,
  created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at    TIMESTAMP NULL,
  CONSTRAINT fk_action_runs_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_action_runs_user FOREIGN KEY (triggered_by) REFERENCES users(id) ON DELETE SET NULL
) ENGINE=InnoDB;

CREATE INDEX idx_action_runs_repository_id ON action_runs(repository_id);
CREATE INDEX idx_action_runs_head_sha ON action_runs(repository_id, head_sha);

CREATE TABLE IF NOT EXISTS action_jobs (
  id            CHAR(36) PRIMARY KEY,
  run_id        CHAR(36) NOT NULL,
  job_key       VARCHAR(255) NOT NULL,
  name          VARCHAR(255) NOT NULL DEFAULT '',
  runs_on_json  TEXT NOT NULL DEFAULT ('[]'),
  status        VARCHAR(32) NOT NULL DEFAULT 'queued',
  runner_id     CHAR(36) NULL,
  started_at    TIMESTAMP NULL,
  finished_at   TIMESTAMP NULL,
  created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uq_action_jobs_run_job (run_id, job_key),
  CONSTRAINT fk_action_jobs_run FOREIGN KEY (run_id) REFERENCES action_runs(id) ON DELETE CASCADE,
  CONSTRAINT fk_action_jobs_runner FOREIGN KEY (runner_id) REFERENCES action_runners(id) ON DELETE SET NULL
) ENGINE=InnoDB;

CREATE INDEX idx_action_jobs_run_id ON action_jobs(run_id);
CREATE INDEX idx_action_jobs_status ON action_jobs(status);

CREATE TABLE IF NOT EXISTS action_secrets (
  id             CHAR(36) PRIMARY KEY,
  repository_id  CHAR(36) NOT NULL,
  name           VARCHAR(255) NOT NULL,
  ciphertext     TEXT NOT NULL,
  created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uq_action_secrets_repo_name (repository_id, name),
  CONSTRAINT fk_action_secrets_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_action_secrets_repository_id ON action_secrets(repository_id);
