-- logical: 0011_issues — issues domain (D-ISS-01..06, D-ISS-11..13)
-- Per-repo sequential #N via issue_counters; hard-delete does not reclaim.
-- MySQL utf8mb4: VARCHAR max length is 16383; use TEXT for markdown bodies.
-- MySQL 8.0.13+ allows DEFAULT on TEXT (CI uses mysql:8.4).

CREATE TABLE IF NOT EXISTS issues (
  id         CHAR(36)      PRIMARY KEY,
  repo_id    CHAR(36)      NOT NULL,
  number     BIGINT        NOT NULL,
  title      VARCHAR(500)  NOT NULL,
  body       TEXT          NOT NULL DEFAULT (''),
  state      VARCHAR(16)   NOT NULL DEFAULT 'open',
  author_id  CHAR(36)      NOT NULL,
  closed_at  TIMESTAMP     NULL,
  closed_by  CHAR(36)      NULL,
  created_at TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_issues_repo_number (repo_id, number),
  CONSTRAINT fk_issues_repo FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_issues_author FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_issues_closed_by FOREIGN KEY (closed_by) REFERENCES users(id) ON DELETE SET NULL,
  CONSTRAINT issues_state_check CHECK (state IN ('open', 'closed'))
) ENGINE=InnoDB;

CREATE INDEX idx_issues_repo_id ON issues(repo_id);
CREATE INDEX idx_issues_repo_state_updated ON issues(repo_id, state, updated_at);
CREATE INDEX idx_issues_author_id ON issues(author_id);

CREATE TABLE IF NOT EXISTS issue_counters (
  repo_id    CHAR(36) PRIMARY KEY,
  max_number BIGINT   NOT NULL DEFAULT 0,
  CONSTRAINT fk_issue_counters_repo FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS issue_comments (
  id         CHAR(36)       PRIMARY KEY,
  issue_id   CHAR(36)       NOT NULL,
  author_id  CHAR(36)       NOT NULL,
  body       TEXT           NOT NULL DEFAULT (''),
  created_at TIMESTAMP      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_issue_comments_issue FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
  CONSTRAINT fk_issue_comments_author FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_issue_comments_issue_id ON issue_comments(issue_id);

CREATE TABLE IF NOT EXISTS issue_revisions (
  id         CHAR(36)       PRIMARY KEY,
  issue_id   CHAR(36)       NOT NULL,
  editor_id  CHAR(36)       NOT NULL,
  title      VARCHAR(500)   NOT NULL,
  body       TEXT           NOT NULL,
  created_at TIMESTAMP      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_issue_revisions_issue FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
  CONSTRAINT fk_issue_revisions_editor FOREIGN KEY (editor_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_issue_revisions_issue_id ON issue_revisions(issue_id);

CREATE TABLE IF NOT EXISTS comment_revisions (
  id         CHAR(36)       PRIMARY KEY,
  comment_id CHAR(36)       NOT NULL,
  editor_id  CHAR(36)       NOT NULL,
  body       TEXT           NOT NULL,
  created_at TIMESTAMP      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_comment_revisions_comment FOREIGN KEY (comment_id) REFERENCES issue_comments(id) ON DELETE CASCADE,
  CONSTRAINT fk_comment_revisions_editor FOREIGN KEY (editor_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_comment_revisions_comment_id ON comment_revisions(comment_id);

CREATE TABLE IF NOT EXISTS labels (
  id          CHAR(36)      PRIMARY KEY,
  name        VARCHAR(100)  NOT NULL,
  color       CHAR(6)       NOT NULL,
  description VARCHAR(500)  NOT NULL DEFAULT '',
  org_id      CHAR(36)      NULL,
  repo_id     CHAR(36)      NULL,
  created_at  TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at  TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  name_lower  VARCHAR(100)  GENERATED ALWAYS AS (LOWER(name)) STORED,
  CONSTRAINT fk_labels_org FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE,
  CONSTRAINT fk_labels_repo FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT labels_scope_xor_check CHECK (
    (org_id IS NOT NULL AND repo_id IS NULL)
    OR (org_id IS NULL AND repo_id IS NOT NULL)
  ),
  UNIQUE KEY uk_labels_org_name (org_id, name_lower),
  UNIQUE KEY uk_labels_repo_name (repo_id, name_lower)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS repo_hidden_labels (
  repo_id  CHAR(36) NOT NULL,
  label_id CHAR(36) NOT NULL,
  PRIMARY KEY (repo_id, label_id),
  CONSTRAINT fk_repo_hidden_labels_repo FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_repo_hidden_labels_label FOREIGN KEY (label_id) REFERENCES labels(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS issue_labels (
  issue_id CHAR(36) NOT NULL,
  label_id CHAR(36) NOT NULL,
  PRIMARY KEY (issue_id, label_id),
  CONSTRAINT fk_issue_labels_issue FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
  CONSTRAINT fk_issue_labels_label FOREIGN KEY (label_id) REFERENCES labels(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_issue_labels_label_id ON issue_labels(label_id);

CREATE TABLE IF NOT EXISTS issue_assignees (
  issue_id   CHAR(36)  NOT NULL,
  user_id    CHAR(36)  NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (issue_id, user_id),
  CONSTRAINT fk_issue_assignees_issue FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
  CONSTRAINT fk_issue_assignees_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_issue_assignees_user_id ON issue_assignees(user_id);

CREATE TABLE IF NOT EXISTS issue_reactions (
  issue_id   CHAR(36)    NOT NULL,
  user_id    CHAR(36)    NOT NULL,
  content    VARCHAR(16) NOT NULL,
  created_at TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (issue_id, user_id, content),
  CONSTRAINT fk_issue_reactions_issue FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
  CONSTRAINT fk_issue_reactions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT issue_reactions_content_check CHECK (
    content IN ('+1', '-1', 'laugh', 'confused', 'heart', 'hooray', 'rocket', 'eyes')
  )
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS comment_reactions (
  comment_id CHAR(36)    NOT NULL,
  user_id    CHAR(36)    NOT NULL,
  content    VARCHAR(16) NOT NULL,
  created_at TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (comment_id, user_id, content),
  CONSTRAINT fk_comment_reactions_comment FOREIGN KEY (comment_id) REFERENCES issue_comments(id) ON DELETE CASCADE,
  CONSTRAINT fk_comment_reactions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT comment_reactions_content_check CHECK (
    content IN ('+1', '-1', 'laugh', 'confused', 'heart', 'hooray', 'rocket', 'eyes')
  )
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS issue_links (
  id               CHAR(36)     PRIMARY KEY,
  issue_id         CHAR(36)     NOT NULL,
  kind             VARCHAR(16)  NOT NULL,
  target_repo_id   CHAR(36)     NULL,
  target_number    BIGINT       NULL,
  target_opaque_id VARCHAR(64)  NULL,
  title            VARCHAR(500) NULL,
  created_by       CHAR(36)     NOT NULL,
  created_at       TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_issue_links_issue FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
  CONSTRAINT fk_issue_links_target_repo FOREIGN KEY (target_repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_issue_links_created_by FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT issue_links_kind_check CHECK (kind IN ('issue', 'pr_stub'))
) ENGINE=InnoDB;

CREATE INDEX idx_issue_links_issue_id ON issue_links(issue_id);
