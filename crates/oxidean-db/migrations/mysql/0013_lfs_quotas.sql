-- logical: 0013_lfs_quotas — instance LFS limits + link attribution (D-LFS-12/13)
CREATE TABLE IF NOT EXISTS instance_lfs_settings (
  id                    TINYINT PRIMARY KEY CHECK (id = 1),
  max_object_bytes      BIGINT NULL,
  quota_repo_bytes      BIGINT NULL,
  quota_user_bytes      BIGINT NULL,
  updated_at            DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
);

INSERT IGNORE INTO instance_lfs_settings (id) VALUES (1);

ALTER TABLE lfs_object_links ADD COLUMN uploaded_by_user_id VARCHAR(36) NULL;
