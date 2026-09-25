-- logical: 0012_lfs — Git LFS volume store + per-repo enable (D-LFS-01..03, D-LFS-10)
ALTER TABLE repositories ADD COLUMN lfs_enabled TINYINT(1) NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS lfs_objects (
  oid        CHAR(64)   PRIMARY KEY,
  size       BIGINT     NOT NULL,
  created_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS lfs_object_links (
  repository_id CHAR(36)  NOT NULL,
  oid           CHAR(64)  NOT NULL,
  refcount      INT       NOT NULL DEFAULT 1,
  created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (repository_id, oid),
  CONSTRAINT fk_lfs_links_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_lfs_links_oid FOREIGN KEY (oid) REFERENCES lfs_objects(oid) ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE INDEX idx_lfs_object_links_oid ON lfs_object_links(oid);
