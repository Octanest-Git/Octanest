-- logical: 0012_packages — multi-format package registry metadata (D-PKG-02, D-PKG-08, D-PKG-09)
-- MySQL: TEXT cannot carry DEFAULT — use VARCHAR where a default is required.

CREATE TABLE IF NOT EXISTS packages (
  id             CHAR(36)      PRIMARY KEY,
  owner_type     VARCHAR(8)    NOT NULL,
  owner_id       CHAR(36)      NOT NULL,
  name           VARCHAR(255)  NOT NULL,
  format         VARCHAR(16)   NOT NULL,
  visibility     VARCHAR(16)   NOT NULL DEFAULT 'private',
  repository_id  CHAR(36)      NULL,
  description    VARCHAR(2000) NOT NULL DEFAULT '',
  created_at     TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at     TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_packages_owner_name_format (owner_type, owner_id, name, format),
  CONSTRAINT fk_packages_repository FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE SET NULL,
  CONSTRAINT packages_owner_type_check CHECK (owner_type IN ('user', 'org')),
  CONSTRAINT packages_format_check CHECK (format IN ('oci', 'npm', 'generic')),
  CONSTRAINT packages_visibility_check CHECK (visibility IN ('public', 'private'))
) ENGINE=InnoDB;

CREATE INDEX idx_packages_owner ON packages(owner_type, owner_id);
CREATE INDEX idx_packages_repository_id ON packages(repository_id);
CREATE INDEX idx_packages_format ON packages(format);

CREATE TABLE IF NOT EXISTS package_versions (
  id            CHAR(36)       PRIMARY KEY,
  package_id    CHAR(36)       NOT NULL,
  version       VARCHAR(255)   NOT NULL,
  digest        VARCHAR(128)   NULL,
  metadata_json VARCHAR(65535) NOT NULL DEFAULT '{}',
  published_by  CHAR(36)       NULL,
  created_at    TIMESTAMP      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_package_versions_pkg_ver (package_id, version),
  CONSTRAINT fk_package_versions_package FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE,
  CONSTRAINT fk_package_versions_publisher FOREIGN KEY (published_by) REFERENCES users(id) ON DELETE SET NULL
) ENGINE=InnoDB;

CREATE INDEX idx_package_versions_package_id ON package_versions(package_id);
CREATE INDEX idx_package_versions_digest ON package_versions(digest);

CREATE TABLE IF NOT EXISTS package_blobs (
  digest     VARCHAR(128) PRIMARY KEY,
  size_bytes BIGINT       NOT NULL,
  refcount   INT          NOT NULL DEFAULT 0,
  created_at TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT package_blobs_size_check CHECK (size_bytes >= 0),
  CONSTRAINT package_blobs_refcount_check CHECK (refcount >= 0)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS package_blob_refs (
  package_version_id CHAR(36)     NOT NULL,
  blob_digest        VARCHAR(128) NOT NULL,
  role               VARCHAR(64)  NOT NULL DEFAULT 'blob',
  PRIMARY KEY (package_version_id, blob_digest, role),
  CONSTRAINT fk_package_blob_refs_version FOREIGN KEY (package_version_id) REFERENCES package_versions(id) ON DELETE CASCADE,
  CONSTRAINT fk_package_blob_refs_blob FOREIGN KEY (blob_digest) REFERENCES package_blobs(digest) ON DELETE RESTRICT
) ENGINE=InnoDB;

CREATE INDEX idx_package_blob_refs_digest ON package_blob_refs(blob_digest);

CREATE TABLE IF NOT EXISTS package_quota_overrides (
  owner_type VARCHAR(8) NOT NULL,
  owner_id   CHAR(36)   NOT NULL,
  max_bytes  BIGINT     NOT NULL,
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (owner_type, owner_id),
  CONSTRAINT package_quota_owner_type_check CHECK (owner_type IN ('user', 'org')),
  CONSTRAINT package_quota_max_bytes_check CHECK (max_bytes >= 0)
) ENGINE=InnoDB;
