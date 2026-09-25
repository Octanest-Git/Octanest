-- logical: 0022_templates — instance template packs + user template repos (issue #18)

ALTER TABLE repositories ADD COLUMN is_template TINYINT(1) NOT NULL DEFAULT 0;
ALTER TABLE repositories ADD COLUMN created_from_template_repo_id CHAR(36) NULL;

CREATE TABLE IF NOT EXISTS instance_template_packs (
  id                   CHAR(36) PRIMARY KEY,
  slug                 VARCHAR(128) NOT NULL,
  label                VARCHAR(255) NOT NULL,
  `group`              VARCHAR(64) NOT NULL DEFAULT 'Custom',
  description          TEXT NOT NULL,
  default_gitignore    VARCHAR(128) NULL,
  enabled              TINYINT(1) NOT NULL DEFAULT 1,
  byte_size            BIGINT NOT NULL,
  content_digest       VARCHAR(128) NOT NULL,
  uploaded_by_user_id  CHAR(36) NOT NULL,
  created_at           TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at           TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uq_instance_template_packs_slug (slug),
  CONSTRAINT fk_instance_template_packs_user FOREIGN KEY (uploaded_by_user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_instance_template_packs_enabled ON instance_template_packs(enabled);
CREATE INDEX idx_repositories_is_template ON repositories(is_template);
