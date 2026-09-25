-- logical: 0008_pats — personal access tokens (hash-at-rest) + FG selected-repo join
CREATE TABLE IF NOT EXISTS personal_access_tokens (
  id             TEXT    PRIMARY KEY,
  user_id        TEXT    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  kind           TEXT    NOT NULL,
  name           TEXT    NOT NULL,
  token_prefix   TEXT    NOT NULL,
  token_hash     TEXT    NOT NULL UNIQUE,
  scopes_json    TEXT    NULL,
  contents_perm  TEXT    NULL,
  repo_access    TEXT    NULL,
  expires_at     TEXT    NULL,
  revoked_at     TEXT    NULL,
  last_used_at   TEXT    NULL,
  last_used_ip   TEXT    NULL,
  created_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE INDEX IF NOT EXISTS idx_pats_user_id ON personal_access_tokens(user_id);

CREATE TABLE IF NOT EXISTS personal_access_token_repos (
  token_id      TEXT NOT NULL REFERENCES personal_access_tokens(id) ON DELETE CASCADE,
  repository_id TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  PRIMARY KEY (token_id, repository_id)
);

CREATE INDEX IF NOT EXISTS idx_pat_repos_repository_id ON personal_access_token_repos(repository_id);
