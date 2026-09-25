-- logical: 0013_lfs_quotas — instance LFS limits + link attribution (D-LFS-12/13)
CREATE TABLE IF NOT EXISTS instance_lfs_settings (
  id                    INTEGER PRIMARY KEY CHECK (id = 1),
  max_object_bytes      INTEGER,
  quota_repo_bytes      INTEGER,
  quota_user_bytes      INTEGER,
  updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

INSERT OR IGNORE INTO instance_lfs_settings (id) VALUES (1);

ALTER TABLE lfs_object_links ADD COLUMN uploaded_by_user_id TEXT REFERENCES users(id) ON DELETE SET NULL;
