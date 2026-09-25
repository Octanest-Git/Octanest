-- logical: 0012_lfs — Git LFS volume store + per-repo enable (D-LFS-01..03, D-LFS-10)
ALTER TABLE repositories ADD COLUMN lfs_enabled BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE IF NOT EXISTS lfs_objects (
  oid        TEXT        PRIMARY KEY,
  size       BIGINT      NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS lfs_object_links (
  repository_id TEXT        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  oid           TEXT        NOT NULL REFERENCES lfs_objects(oid) ON DELETE CASCADE,
  refcount      INTEGER     NOT NULL DEFAULT 1,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (repository_id, oid)
);

CREATE INDEX IF NOT EXISTS idx_lfs_object_links_oid ON lfs_object_links(oid);
