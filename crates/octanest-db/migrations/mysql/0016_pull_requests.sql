-- logical: 0016_pull_requests — PR domain + merge settings + fork parent (Phase 12)

ALTER TABLE repositories ADD COLUMN allow_merge_commit TINYINT(1) NOT NULL DEFAULT 1;
ALTER TABLE repositories ADD COLUMN allow_squash_merge TINYINT(1) NOT NULL DEFAULT 1;
ALTER TABLE repositories ADD COLUMN allow_rebase_merge TINYINT(1) NOT NULL DEFAULT 1;
ALTER TABLE repositories ADD COLUMN forked_from_repo_id CHAR(36) NULL;

ALTER TABLE repositories
  ADD CONSTRAINT fk_repositories_forked_from
  FOREIGN KEY (forked_from_repo_id) REFERENCES repositories(id) ON DELETE SET NULL;

CREATE INDEX idx_repositories_forked_from ON repositories(forked_from_repo_id);

CREATE TABLE IF NOT EXISTS pull_requests (
  id               CHAR(36)      PRIMARY KEY,
  repo_id          CHAR(36)      NOT NULL,
  number           BIGINT        NOT NULL,
  title            VARCHAR(500)  NOT NULL,
  body             TEXT          NOT NULL DEFAULT (''),
  state            VARCHAR(16)   NOT NULL DEFAULT 'open',
  draft            TINYINT(1)    NOT NULL DEFAULT 0,
  author_id        CHAR(36)      NOT NULL,
  base_ref         VARCHAR(255)  NOT NULL,
  base_sha         VARCHAR(64)   NOT NULL DEFAULT '',
  head_repo_id     CHAR(36)      NOT NULL,
  head_ref         VARCHAR(255)  NOT NULL,
  head_sha         VARCHAR(64)   NOT NULL DEFAULT '',
  merged_at        TIMESTAMP     NULL,
  merged_by        CHAR(36)      NULL,
  merge_commit_sha VARCHAR(64)   NULL,
  merge_method     VARCHAR(16)   NULL,
  closed_at        TIMESTAMP     NULL,
  closed_by        CHAR(36)      NULL,
  created_at       TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at       TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_pull_requests_repo_number (repo_id, number),
  CONSTRAINT fk_pull_requests_repo FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_pull_requests_author FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_pull_requests_head_repo FOREIGN KEY (head_repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_pull_requests_merged_by FOREIGN KEY (merged_by) REFERENCES users(id) ON DELETE SET NULL,
  CONSTRAINT fk_pull_requests_closed_by FOREIGN KEY (closed_by) REFERENCES users(id) ON DELETE SET NULL,
  CONSTRAINT pull_requests_state_check CHECK (state IN ('open', 'closed', 'merged')),
  CONSTRAINT pull_requests_merge_method_check
    CHECK (merge_method IS NULL OR merge_method IN ('merge', 'squash', 'rebase'))
) ENGINE=InnoDB;

CREATE INDEX idx_pull_requests_repo_id ON pull_requests(repo_id);
CREATE INDEX idx_pull_requests_repo_state_updated ON pull_requests(repo_id, state, updated_at);
CREATE INDEX idx_pull_requests_author_id ON pull_requests(author_id);
CREATE INDEX idx_pull_requests_head_repo_id ON pull_requests(head_repo_id);

CREATE TABLE IF NOT EXISTS pull_comments (
  id         CHAR(36)      PRIMARY KEY,
  pull_id    CHAR(36)      NOT NULL,
  author_id  CHAR(36)      NOT NULL,
  body       TEXT          NOT NULL DEFAULT (''),
  path       VARCHAR(1024) NULL,
  side       VARCHAR(8)    NULL,
  line       BIGINT        NULL,
  start_line BIGINT        NULL,
  commit_sha VARCHAR(64)   NULL,
  outdated   TINYINT(1)    NOT NULL DEFAULT 0,
  resolved   TINYINT(1)    NOT NULL DEFAULT 0,
  review_id  CHAR(36)      NULL,
  created_at TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_pull_comments_pull FOREIGN KEY (pull_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
  CONSTRAINT fk_pull_comments_author FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT pull_comments_side_check CHECK (side IS NULL OR side IN ('LEFT', 'RIGHT'))
) ENGINE=InnoDB;

CREATE INDEX idx_pull_comments_pull_id ON pull_comments(pull_id);

CREATE TABLE IF NOT EXISTS pull_reviews (
  id             CHAR(36)     PRIMARY KEY,
  pull_id        CHAR(36)     NOT NULL,
  author_id      CHAR(36)     NOT NULL,
  state          VARCHAR(32)  NOT NULL,
  body           TEXT         NOT NULL DEFAULT (''),
  commit_sha     VARCHAR(64)  NULL,
  submitted_at   TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  dismissed_at   TIMESTAMP    NULL,
  dismiss_reason VARCHAR(500) NULL,
  CONSTRAINT fk_pull_reviews_pull FOREIGN KEY (pull_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
  CONSTRAINT fk_pull_reviews_author FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT pull_reviews_state_check
    CHECK (state IN ('approved', 'changes_requested', 'commented', 'dismissed'))
) ENGINE=InnoDB;

CREATE INDEX idx_pull_reviews_pull_id ON pull_reviews(pull_id);
CREATE INDEX idx_pull_reviews_author ON pull_reviews(pull_id, author_id, submitted_at);

CREATE TABLE IF NOT EXISTS pull_review_requests (
  pull_id      CHAR(36) NOT NULL,
  user_id      CHAR(36) NOT NULL,
  requested_by CHAR(36) NOT NULL,
  created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (pull_id, user_id),
  CONSTRAINT fk_pull_review_requests_pull FOREIGN KEY (pull_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
  CONSTRAINT fk_pull_review_requests_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_pull_review_requests_by FOREIGN KEY (requested_by) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS pull_labels (
  pull_id  CHAR(36) NOT NULL,
  label_id CHAR(36) NOT NULL,
  PRIMARY KEY (pull_id, label_id),
  CONSTRAINT fk_pull_labels_pull FOREIGN KEY (pull_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
  CONSTRAINT fk_pull_labels_label FOREIGN KEY (label_id) REFERENCES labels(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS pull_assignees (
  pull_id CHAR(36) NOT NULL,
  user_id CHAR(36) NOT NULL,
  PRIMARY KEY (pull_id, user_id),
  CONSTRAINT fk_pull_assignees_pull FOREIGN KEY (pull_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
  CONSTRAINT fk_pull_assignees_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

-- Expand issue_links kind CHECK (MySQL 8.0.19+ ALTER CHECK / drop by name).
ALTER TABLE issue_links DROP CHECK issue_links_kind_check;
ALTER TABLE issue_links ADD CONSTRAINT issue_links_kind_check
  CHECK (kind IN ('issue', 'pr_stub', 'pr'));
