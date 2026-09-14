-- logical: 0008_pats — personal access tokens (hash-at-rest) + FG selected-repo join
-- MySQL: TEXT cannot carry DEFAULT — use VARCHAR where a default is required.
CREATE TABLE IF NOT EXISTS personal_access_tokens (
  id             CHAR(36)     PRIMARY KEY,
  user_id        CHAR(36)     NOT NULL,
  kind           VARCHAR(32)  NOT NULL,
  name           VARCHAR(200) NOT NULL,
  token_prefix   VARCHAR(64)  NOT NULL,
  token_hash     CHAR(64)     NOT NULL,
  scopes_json    TEXT         NULL,
  contents_perm  VARCHAR(16)  NULL,
  repo_access    VARCHAR(16)  NULL,
  expires_at     TIMESTAMP    NULL,
  revoked_at     TIMESTAMP    NULL,
  last_used_at   TIMESTAMP    NULL,
  last_used_ip   VARCHAR(64)  NULL,
  created_at     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_pats_token_hash (token_hash),
  CONSTRAINT fk_pats_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_pats_user_id ON personal_access_tokens(user_id);

CREATE TABLE IF NOT EXISTS personal_access_token_repos (
  token_id      CHAR(36) NOT NULL,
  repository_id CHAR(36) NOT NULL,
  PRIMARY KEY (token_id, repository_id),
  CONSTRAINT fk_pat_repos_token FOREIGN KEY (token_id) REFERENCES personal_access_tokens(id) ON DELETE CASCADE,
  CONSTRAINT fk_pat_repos_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_pat_repos_repository_id ON personal_access_token_repos(repository_id);
