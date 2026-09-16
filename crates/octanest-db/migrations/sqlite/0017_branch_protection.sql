-- logical: 0017_branch_protection — classic rules + commit statuses (Phase 13 / ORG-05/06 / PR-08)
-- Dialect SQL only.

CREATE TABLE IF NOT EXISTS branch_protection_rules (
  id                              TEXT PRIMARY KEY,
  repo_id                         TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  pattern                         TEXT NOT NULL,
  require_reviews                 INTEGER NOT NULL DEFAULT 0,
  required_approving_review_count INTEGER NOT NULL DEFAULT 1,
  dismiss_stale_reviews           INTEGER NOT NULL DEFAULT 0,
  require_conversation_resolution INTEGER NOT NULL DEFAULT 0,
  require_last_push_approval      INTEGER NOT NULL DEFAULT 0,
  required_status_contexts        TEXT NOT NULL DEFAULT '[]',
  strict_status_checks            INTEGER NOT NULL DEFAULT 0,
  allow_force_pushes              INTEGER NOT NULL DEFAULT 0,
  allow_deletions                 INTEGER NOT NULL DEFAULT 0,
  enforce_admins                  INTEGER NOT NULL DEFAULT 0,
  required_linear_history         INTEGER NOT NULL DEFAULT 0,
  lock_branch                     INTEGER NOT NULL DEFAULT 0,
  created_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  updated_at                      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
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
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  UNIQUE (repo_id, sha, context)
);

CREATE INDEX IF NOT EXISTS idx_commit_statuses_repo_sha
  ON commit_statuses(repo_id, sha);
