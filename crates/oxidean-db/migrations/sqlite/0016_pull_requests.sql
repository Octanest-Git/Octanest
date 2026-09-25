-- logical: 0016_pull_requests — PR domain + merge settings + fork parent (Phase 12)
-- Shared #N via issue_counters (D-PR-02). Dialect SQL only.

ALTER TABLE repositories ADD COLUMN allow_merge_commit INTEGER NOT NULL DEFAULT 1;
ALTER TABLE repositories ADD COLUMN allow_squash_merge INTEGER NOT NULL DEFAULT 1;
ALTER TABLE repositories ADD COLUMN allow_rebase_merge INTEGER NOT NULL DEFAULT 1;
ALTER TABLE repositories ADD COLUMN forked_from_repo_id TEXT NULL REFERENCES repositories(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_repositories_forked_from
  ON repositories(forked_from_repo_id);

CREATE TABLE IF NOT EXISTS pull_requests (
  id              TEXT PRIMARY KEY,
  repo_id         TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  number          INTEGER NOT NULL,
  title           TEXT NOT NULL,
  body            TEXT NOT NULL DEFAULT '',
  state           TEXT NOT NULL DEFAULT 'open'
    CHECK (state IN ('open', 'closed', 'merged')),
  draft           INTEGER NOT NULL DEFAULT 0,
  author_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  base_ref        TEXT NOT NULL,
  base_sha        TEXT NOT NULL DEFAULT '',
  head_repo_id    TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  head_ref        TEXT NOT NULL,
  head_sha        TEXT NOT NULL DEFAULT '',
  merged_at       TEXT NULL,
  merged_by       TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
  merge_commit_sha TEXT NULL,
  merge_method    TEXT NULL
    CHECK (merge_method IS NULL OR merge_method IN ('merge', 'squash', 'rebase')),
  closed_at       TEXT NULL,
  closed_by       TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
  created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  UNIQUE (repo_id, number)
);

CREATE INDEX IF NOT EXISTS idx_pull_requests_repo_id ON pull_requests(repo_id);
CREATE INDEX IF NOT EXISTS idx_pull_requests_repo_state_updated
  ON pull_requests(repo_id, state, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_pull_requests_author_id ON pull_requests(author_id);
CREATE INDEX IF NOT EXISTS idx_pull_requests_head_repo_id ON pull_requests(head_repo_id);

CREATE TABLE IF NOT EXISTS pull_comments (
  id           TEXT PRIMARY KEY,
  pull_id      TEXT NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  author_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  body         TEXT NOT NULL DEFAULT '',
  -- NULL path = general conversation comment
  path         TEXT NULL,
  side         TEXT NULL CHECK (side IS NULL OR side IN ('LEFT', 'RIGHT')),
  line         INTEGER NULL,
  start_line   INTEGER NULL,
  commit_sha   TEXT NULL,
  outdated     INTEGER NOT NULL DEFAULT 0,
  resolved     INTEGER NOT NULL DEFAULT 0,
  review_id    TEXT NULL,
  created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE INDEX IF NOT EXISTS idx_pull_comments_pull_id ON pull_comments(pull_id);

CREATE TABLE IF NOT EXISTS pull_reviews (
  id           TEXT PRIMARY KEY,
  pull_id      TEXT NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  author_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  state        TEXT NOT NULL
    CHECK (state IN ('approved', 'changes_requested', 'commented', 'dismissed')),
  body         TEXT NOT NULL DEFAULT '',
  commit_sha   TEXT NULL,
  submitted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  dismissed_at TEXT NULL,
  dismiss_reason TEXT NULL
);

CREATE INDEX IF NOT EXISTS idx_pull_reviews_pull_id ON pull_reviews(pull_id);
CREATE INDEX IF NOT EXISTS idx_pull_reviews_author ON pull_reviews(pull_id, author_id, submitted_at DESC);

CREATE TABLE IF NOT EXISTS pull_review_requests (
  pull_id    TEXT NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  requested_by TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  PRIMARY KEY (pull_id, user_id)
);

CREATE TABLE IF NOT EXISTS pull_labels (
  pull_id  TEXT NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
  PRIMARY KEY (pull_id, label_id)
);

CREATE TABLE IF NOT EXISTS pull_assignees (
  pull_id TEXT NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  PRIMARY KEY (pull_id, user_id)
);

-- Expand issue_links kind to include real PRs (keep pr_stub for legacy rows).
PRAGMA foreign_keys = OFF;
CREATE TABLE issue_links_new (
  id               TEXT PRIMARY KEY,
  issue_id         TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  kind             TEXT NOT NULL CHECK (kind IN ('issue', 'pr_stub', 'pr')),
  target_repo_id   TEXT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  target_number    INTEGER NULL,
  target_opaque_id TEXT NULL,
  title            TEXT NULL,
  created_by       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);
INSERT INTO issue_links_new
  SELECT id, issue_id, kind, target_repo_id, target_number, target_opaque_id, title, created_by, created_at
  FROM issue_links;
DROP TABLE issue_links;
ALTER TABLE issue_links_new RENAME TO issue_links;
CREATE INDEX IF NOT EXISTS idx_issue_links_issue_id ON issue_links(issue_id);
PRAGMA foreign_keys = ON;
