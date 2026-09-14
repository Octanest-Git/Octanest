-- logical: 0013_lfs_quotas — instance LFS limits + link attribution (D-LFS-12/13)
CREATE TABLE IF NOT EXISTS instance_lfs_settings (
  id                    SMALLINT PRIMARY KEY CHECK (id = 1),
  max_object_bytes      BIGINT,
  quota_repo_bytes      BIGINT,
  quota_user_bytes      BIGINT,
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO instance_lfs_settings (id) VALUES (1) ON CONFLICT (id) DO NOTHING;

ALTER TABLE lfs_object_links ADD COLUMN IF NOT EXISTS uploaded_by_user_id TEXT REFERENCES users(id) ON DELETE SET NULL;
