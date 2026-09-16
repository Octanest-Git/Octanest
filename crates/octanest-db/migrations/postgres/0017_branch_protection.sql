-- logical: 0017_branch_protection — classic rules + commit statuses (Phase 13)

CREATE TABLE IF NOT EXISTS branch_protection_rules (
  id                              TEXT PRIMARY KEY,
  repo_id                         TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  pattern                         TEXT NOT NULL,
  require_reviews                 BOOLEAN NOT NULL DEFAULT false,
  required_approving_review_count INTEGER NOT NULL DEFAULT 1,
  dismiss_stale_reviews           BOOLEAN NOT NULL DEFAULT false,
  require_conversation_resolution BOOLEAN NOT NULL DEFAULT false,
  require_last_push_approval      BOOLEAN NOT NULL DEFAULT false,
  required_status_contexts        TEXT NOT NULL DEFAULT '[]',
  strict_status_checks            BOOLEAN NOT NULL DEFAULT false,
  allow_force_pushes              BOOLEAN NOT NULL DEFAULT false,
  allow_deletions                 BOOLEAN NOT NULL DEFAULT false,
  enforce_admins                  BOOLEAN NOT NULL DEFAULT false,
  required_linear_history         BOOLEAN NOT NULL DEFAULT false,
  lock_branch                     BOOLEAN NOT NULL DEFAULT false,
  created_at                      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at                      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_branch_protection_rules_repo
  ON branch_protection_rules(repo_id);

CREATE TABLE IF NOT EXISTS commit_statuses (
  id           TEXT PRIMARY KEY,
  repo_id      TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  sha          TEXT NOT NULL,
  context      TEXT NOT NULL,
  state        TEXT NOT NULL
    CHECK (state IN ('pending', 'success', 'failure', 'error')),
  description  TEXT NOT NULL DEFAULT '',
  target_url   TEXT NULL,
  creator_id   TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (repo_id, sha, context)
);

CREATE INDEX IF NOT EXISTS idx_commit_statuses_repo_sha
  ON commit_statuses(repo_id, sha);
