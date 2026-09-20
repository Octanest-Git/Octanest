-- logical: 0026_repo_mirrors — two-way repository mirroring (GIT-V2-01)

CREATE TABLE IF NOT EXISTS repository_mirrors (
  id                         TEXT PRIMARY KEY,
  repository_id              TEXT NOT NULL UNIQUE REFERENCES repositories(id) ON DELETE CASCADE,
  remote_url                 TEXT NOT NULL,
  auth_kind                  TEXT NOT NULL CHECK (auth_kind IN ('https_token', 'ssh_key')),
  username                   TEXT NOT NULL DEFAULT '',
  secret_ciphertext          TEXT NOT NULL DEFAULT '',
  ssh_public_key             TEXT NOT NULL DEFAULT '',
  known_hosts                TEXT NOT NULL DEFAULT '',
  webhook_secret_ciphertext  TEXT NOT NULL DEFAULT '',
  poll_interval_secs         INTEGER NOT NULL DEFAULT 60,
  enabled                    BOOLEAN NOT NULL DEFAULT TRUE,
  last_synced_at             TIMESTAMPTZ NULL,
  last_status                TEXT NOT NULL DEFAULT 'never'
    CHECK (last_status IN ('never', 'ok', 'error', 'conflict', 'running')),
  last_error                 TEXT NOT NULL DEFAULT '',
  created_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_repository_mirrors_enabled
  ON repository_mirrors(enabled);

CREATE TABLE IF NOT EXISTS repository_mirror_ref_results (
  id              TEXT PRIMARY KEY,
  mirror_id       TEXT NOT NULL REFERENCES repository_mirrors(id) ON DELETE CASCADE,
  refname         TEXT NOT NULL,
  outcome         TEXT NOT NULL
    CHECK (outcome IN ('ff_in', 'ff_out', 'merged', 'conflict', 'skipped', 'error')),
  local_oid       TEXT NOT NULL DEFAULT '',
  remote_oid      TEXT NOT NULL DEFAULT '',
  detail          TEXT NOT NULL DEFAULT '',
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (mirror_id, refname)
);

CREATE INDEX IF NOT EXISTS idx_repository_mirror_ref_results_mirror
  ON repository_mirror_ref_results(mirror_id);
