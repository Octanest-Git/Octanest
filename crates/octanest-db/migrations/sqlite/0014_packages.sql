-- logical: 0014_packages — multi-format package registry metadata (D-PKG-02, D-PKG-08, D-PKG-09)

CREATE TABLE IF NOT EXISTS packages (
  id             TEXT PRIMARY KEY,
  owner_type     TEXT NOT NULL CHECK (owner_type IN ('user', 'org')),
  owner_id       TEXT NOT NULL,
  name           TEXT NOT NULL,
  format         TEXT NOT NULL CHECK (format IN ('oci', 'npm', 'generic')),
  visibility     TEXT NOT NULL DEFAULT 'private'
    CHECK (visibility IN ('public', 'private')),
  repository_id  TEXT NULL REFERENCES repositories(id) ON DELETE SET NULL,
  description    TEXT NOT NULL DEFAULT '',
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  UNIQUE (owner_type, owner_id, name, format)
);

CREATE INDEX IF NOT EXISTS idx_packages_owner ON packages(owner_type, owner_id);
CREATE INDEX IF NOT EXISTS idx_packages_repository_id ON packages(repository_id);
CREATE INDEX IF NOT EXISTS idx_packages_format ON packages(format);

CREATE TABLE IF NOT EXISTS package_versions (
  id            TEXT PRIMARY KEY,
  package_id    TEXT NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
  version       TEXT NOT NULL,
  digest        TEXT NULL,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  published_by  TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
  created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  UNIQUE (package_id, version)
);

CREATE INDEX IF NOT EXISTS idx_package_versions_package_id ON package_versions(package_id);
CREATE INDEX IF NOT EXISTS idx_package_versions_digest ON package_versions(digest);

CREATE TABLE IF NOT EXISTS package_blobs (
  digest     TEXT PRIMARY KEY,
  size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
  refcount   INTEGER NOT NULL DEFAULT 0 CHECK (refcount >= 0),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE TABLE IF NOT EXISTS package_blob_refs (
  package_version_id TEXT NOT NULL REFERENCES package_versions(id) ON DELETE CASCADE,
  blob_digest        TEXT NOT NULL REFERENCES package_blobs(digest) ON DELETE RESTRICT,
  role               TEXT NOT NULL DEFAULT 'blob',
  PRIMARY KEY (package_version_id, blob_digest, role)
);

CREATE INDEX IF NOT EXISTS idx_package_blob_refs_digest ON package_blob_refs(blob_digest);

CREATE TABLE IF NOT EXISTS package_quota_overrides (
  owner_type TEXT NOT NULL CHECK (owner_type IN ('user', 'org')),
  owner_id   TEXT NOT NULL,
  max_bytes  INTEGER NOT NULL CHECK (max_bytes >= 0),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  PRIMARY KEY (owner_type, owner_id)
);
