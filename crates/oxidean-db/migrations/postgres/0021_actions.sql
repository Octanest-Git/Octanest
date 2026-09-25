-- logical: 0021_actions — Actions runners/runs/jobs/secrets + repo gate (D-ACT-13, D-ACT-17)
-- commit_statuses already from 0017_branch_protection (Phase 13 / D-ACT-15 consumer).

ALTER TABLE repositories ADD COLUMN actions_enabled BOOLEAN NOT NULL DEFAULT true;

CREATE TABLE IF NOT EXISTS action_runners (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  token_hash    TEXT NOT NULL,
  labels_json   TEXT NOT NULL DEFAULT '[]',
  owner_type    TEXT NULL CHECK (owner_type IS NULL OR owner_type IN ('instance', 'org', 'user', 'repo')),
  owner_id      TEXT NULL,
  repository_id TEXT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  ephemeral     BOOLEAN NOT NULL DEFAULT false,
  last_online   TIMESTAMPTZ NULL,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_action_runners_repository_id ON action_runners(repository_id);
CREATE INDEX IF NOT EXISTS idx_action_runners_token_hash ON action_runners(token_hash);

CREATE TABLE IF NOT EXISTS action_runner_tokens (
  id            TEXT PRIMARY KEY,
  token_hash    TEXT NOT NULL UNIQUE,
  scope_type    TEXT NOT NULL CHECK (scope_type IN ('instance', 'org', 'repo')),
  scope_id      TEXT NULL,
  active        BOOLEAN NOT NULL DEFAULT true,
  created_by    TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at    TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_action_runner_tokens_active ON action_runner_tokens(active);

CREATE TABLE IF NOT EXISTS action_runs (
  id             TEXT PRIMARY KEY,
  repository_id  TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  workflow_path  TEXT NOT NULL,
  workflow_name  TEXT NOT NULL DEFAULT '',
  event          TEXT NOT NULL,
  head_sha       TEXT NOT NULL,
  head_ref       TEXT NOT NULL DEFAULT '',
  status         TEXT NOT NULL DEFAULT 'queued'
    CHECK (status IN ('queued', 'in_progress', 'success', 'failure', 'cancelled')),
  title          TEXT NOT NULL DEFAULT '',
  triggered_by   TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  finished_at    TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_action_runs_repository_id ON action_runs(repository_id);
CREATE INDEX IF NOT EXISTS idx_action_runs_head_sha ON action_runs(repository_id, head_sha);

CREATE TABLE IF NOT EXISTS action_jobs (
  id            TEXT PRIMARY KEY,
  run_id        TEXT NOT NULL REFERENCES action_runs(id) ON DELETE CASCADE,
  job_key       TEXT NOT NULL,
  name          TEXT NOT NULL DEFAULT '',
  runs_on_json  TEXT NOT NULL DEFAULT '[]',
  status        TEXT NOT NULL DEFAULT 'queued'
    CHECK (status IN ('queued', 'in_progress', 'success', 'failure', 'cancelled')),
  runner_id     TEXT NULL REFERENCES action_runners(id) ON DELETE SET NULL,
  started_at    TIMESTAMPTZ NULL,
  finished_at   TIMESTAMPTZ NULL,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT action_jobs_run_job_unique UNIQUE (run_id, job_key)
);

CREATE INDEX IF NOT EXISTS idx_action_jobs_run_id ON action_jobs(run_id);
CREATE INDEX IF NOT EXISTS idx_action_jobs_status ON action_jobs(status);

CREATE TABLE IF NOT EXISTS action_secrets (
  id             TEXT PRIMARY KEY,
  repository_id  TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  name           TEXT NOT NULL,
  ciphertext     TEXT NOT NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT action_secrets_repo_name_unique UNIQUE (repository_id, name)
);

CREATE INDEX IF NOT EXISTS idx_action_secrets_repository_id ON action_secrets(repository_id);
