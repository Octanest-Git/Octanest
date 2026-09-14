-- logical: 0011_issues — issues domain (D-ISS-01..06, D-ISS-11..13)
-- Per-repo sequential #N via issue_counters; hard-delete does not reclaim.

CREATE TABLE IF NOT EXISTS issues (
  id         TEXT        PRIMARY KEY,
  repo_id    TEXT        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  number     BIGINT      NOT NULL,
  title      TEXT        NOT NULL,
  body       TEXT        NOT NULL DEFAULT '',
  state      TEXT        NOT NULL DEFAULT 'open',
  author_id  TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  closed_at  TIMESTAMPTZ NULL,
  closed_by  TEXT        NULL REFERENCES users(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT issues_state_check CHECK (state IN ('open', 'closed')),
  CONSTRAINT issues_repo_number_unique UNIQUE (repo_id, number)
);

CREATE INDEX IF NOT EXISTS idx_issues_repo_id ON issues(repo_id);
CREATE INDEX IF NOT EXISTS idx_issues_repo_state_updated
  ON issues(repo_id, state, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_issues_author_id ON issues(author_id);

CREATE TABLE IF NOT EXISTS issue_counters (
  repo_id    TEXT   PRIMARY KEY REFERENCES repositories(id) ON DELETE CASCADE,
  max_number BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS issue_comments (
  id         TEXT        PRIMARY KEY,
  issue_id   TEXT        NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  author_id  TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  body       TEXT        NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_issue_comments_issue_id ON issue_comments(issue_id);

CREATE TABLE IF NOT EXISTS issue_revisions (
  id         TEXT        PRIMARY KEY,
  issue_id   TEXT        NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  editor_id  TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  title      TEXT        NOT NULL,
  body       TEXT        NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_issue_revisions_issue_id ON issue_revisions(issue_id);

CREATE TABLE IF NOT EXISTS comment_revisions (
  id         TEXT        PRIMARY KEY,
  comment_id TEXT        NOT NULL REFERENCES issue_comments(id) ON DELETE CASCADE,
  editor_id  TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  body       TEXT        NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_comment_revisions_comment_id ON comment_revisions(comment_id);

-- Dual-scope labels: exactly one of org_id / repo_id (D-ISS-05).
CREATE TABLE IF NOT EXISTS labels (
  id          TEXT        PRIMARY KEY,
  name        TEXT        NOT NULL,
  color       VARCHAR(6)  NOT NULL,
  description TEXT        NOT NULL DEFAULT '',
  org_id      TEXT        NULL REFERENCES organizations(id) ON DELETE CASCADE,
  repo_id     TEXT        NULL REFERENCES repositories(id) ON DELETE CASCADE,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT labels_scope_xor_check CHECK (
    (org_id IS NOT NULL AND repo_id IS NULL)
    OR (org_id IS NULL AND repo_id IS NOT NULL)
  )
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_labels_org_name_lower
  ON labels (org_id, lower(name))
  WHERE org_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_labels_repo_name_lower
  ON labels (repo_id, lower(name))
  WHERE repo_id IS NOT NULL;

-- Repo may hide inherited org labels (inherit + hide + local-only).
CREATE TABLE IF NOT EXISTS repo_hidden_labels (
  repo_id  TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
  PRIMARY KEY (repo_id, label_id)
);

CREATE TABLE IF NOT EXISTS issue_labels (
  issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
  PRIMARY KEY (issue_id, label_id)
);

CREATE INDEX IF NOT EXISTS idx_issue_labels_label_id ON issue_labels(label_id);

CREATE TABLE IF NOT EXISTS issue_assignees (
  issue_id   TEXT        NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  user_id    TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (issue_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_issue_assignees_user_id ON issue_assignees(user_id);

CREATE TABLE IF NOT EXISTS issue_reactions (
  issue_id   TEXT        NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  user_id    TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  content    TEXT        NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (issue_id, user_id, content),
  CONSTRAINT issue_reactions_content_check CHECK (
    content IN ('+1', '-1', 'laugh', 'confused', 'heart', 'hooray', 'rocket', 'eyes')
  )
);

CREATE TABLE IF NOT EXISTS comment_reactions (
  comment_id TEXT        NOT NULL REFERENCES issue_comments(id) ON DELETE CASCADE,
  user_id    TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  content    TEXT        NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (comment_id, user_id, content),
  CONSTRAINT comment_reactions_content_check CHECK (
    content IN ('+1', '-1', 'laugh', 'confused', 'heart', 'hooray', 'rocket', 'eyes')
  )
);

CREATE TABLE IF NOT EXISTS issue_links (
  id               TEXT        PRIMARY KEY,
  issue_id         TEXT        NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  kind             TEXT        NOT NULL,
  target_repo_id   TEXT        NULL REFERENCES repositories(id) ON DELETE CASCADE,
  target_number    BIGINT      NULL,
  target_opaque_id TEXT        NULL,
  title            TEXT        NULL,
  created_by       TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT issue_links_kind_check CHECK (kind IN ('issue', 'pr_stub'))
);

CREATE INDEX IF NOT EXISTS idx_issue_links_issue_id ON issue_links(issue_id);
