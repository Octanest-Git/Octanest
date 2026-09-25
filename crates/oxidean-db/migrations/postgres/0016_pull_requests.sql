-- logical: 0016_pull_requests — PR domain + merge settings + fork parent (Phase 12)

ALTER TABLE repositories ADD COLUMN allow_merge_commit BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE repositories ADD COLUMN allow_squash_merge BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE repositories ADD COLUMN allow_rebase_merge BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE repositories ADD COLUMN forked_from_repo_id TEXT NULL REFERENCES repositories(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_repositories_forked_from
  ON repositories(forked_from_repo_id);

CREATE TABLE IF NOT EXISTS pull_requests (
  id               TEXT PRIMARY KEY,
  repo_id          TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  number           BIGINT NOT NULL,
  title            TEXT NOT NULL,
  body             TEXT NOT NULL DEFAULT '',
  state            TEXT NOT NULL DEFAULT 'open'
    CHECK (state IN ('open', 'closed', 'merged')),
  draft            BOOLEAN NOT NULL DEFAULT false,
  author_id        TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  base_ref         TEXT NOT NULL,
  base_sha         TEXT NOT NULL DEFAULT '',
  head_repo_id     TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  head_ref         TEXT NOT NULL,
  head_sha         TEXT NOT NULL DEFAULT '',
  merged_at        TIMESTAMPTZ NULL,
  merged_by        TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
  merge_commit_sha TEXT NULL,
  merge_method     TEXT NULL
    CHECK (merge_method IS NULL OR merge_method IN ('merge', 'squash', 'rebase')),
  closed_at        TIMESTAMPTZ NULL,
  closed_by        TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (repo_id, number)
);

CREATE INDEX IF NOT EXISTS idx_pull_requests_repo_id ON pull_requests(repo_id);
CREATE INDEX IF NOT EXISTS idx_pull_requests_repo_state_updated
  ON pull_requests(repo_id, state, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_pull_requests_author_id ON pull_requests(author_id);
CREATE INDEX IF NOT EXISTS idx_pull_requests_head_repo_id ON pull_requests(head_repo_id);

CREATE TABLE IF NOT EXISTS pull_comments (
  id         TEXT PRIMARY KEY,
  pull_id    TEXT NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  author_id  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  body       TEXT NOT NULL DEFAULT '',
  path       TEXT NULL,
  side       TEXT NULL CHECK (side IS NULL OR side IN ('LEFT', 'RIGHT')),
  line       BIGINT NULL,
  start_line BIGINT NULL,
  commit_sha TEXT NULL,
  outdated   BOOLEAN NOT NULL DEFAULT false,
  resolved   BOOLEAN NOT NULL DEFAULT false,
  review_id  TEXT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_pull_comments_pull_id ON pull_comments(pull_id);

CREATE TABLE IF NOT EXISTS pull_reviews (
  id             TEXT PRIMARY KEY,
  pull_id        TEXT NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  author_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  state          TEXT NOT NULL
    CHECK (state IN ('approved', 'changes_requested', 'commented', 'dismissed')),
  body           TEXT NOT NULL DEFAULT '',
  commit_sha     TEXT NULL,
  submitted_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  dismissed_at   TIMESTAMPTZ NULL,
  dismiss_reason TEXT NULL
);

CREATE INDEX IF NOT EXISTS idx_pull_reviews_pull_id ON pull_reviews(pull_id);
CREATE INDEX IF NOT EXISTS idx_pull_reviews_author
  ON pull_reviews(pull_id, author_id, submitted_at DESC);

CREATE TABLE IF NOT EXISTS pull_review_requests (
  pull_id      TEXT NOT NULL REFERENCES pull_requests(id) ON DELETE CASCADE,
  user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  requested_by TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
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

ALTER TABLE issue_links DROP CONSTRAINT IF EXISTS issue_links_kind_check;
ALTER TABLE issue_links ADD CONSTRAINT issue_links_kind_check
  CHECK (kind IN ('issue', 'pr_stub', 'pr'));
