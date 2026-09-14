-- logical: 0010_orgs_acl — organizations + ACL tables + polymorphic repository owner
-- Phase 09 owns 0009_ssh_keys; orgs use 0010 (D-ORG-01).
-- SQLite cannot DROP FK in place — rebuild repositories without user FK.

CREATE TABLE IF NOT EXISTS organizations (
  id                     TEXT    PRIMARY KEY,
  slug                   TEXT    NOT NULL UNIQUE,
  display_name           TEXT    NOT NULL DEFAULT '',
  member_base_permission TEXT    NOT NULL DEFAULT 'none'
    CHECK (member_base_permission IN ('none', 'read', 'write')),
  created_at             TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  updated_at             TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_organizations_slug_lower
  ON organizations (lower(slug));

CREATE TABLE IF NOT EXISTS organization_members (
  org_id     TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role       TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  PRIMARY KEY (org_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_organization_members_user_id
  ON organization_members(user_id);

CREATE TABLE IF NOT EXISTS organization_invites (
  id          TEXT PRIMARY KEY,
  org_id      TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  email       TEXT NOT NULL,
  role        TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
  token_hash  TEXT NOT NULL UNIQUE,
  expires_at  TEXT NOT NULL,
  invited_by  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  accepted_at TEXT NULL,
  revoked_at  TEXT NULL
);

CREATE INDEX IF NOT EXISTS idx_organization_invites_org_id
  ON organization_invites(org_id);

-- Rebuild repositories first (add owner_type, drop owner_id → users FK), then collaborators.
PRAGMA foreign_keys = OFF;

CREATE TABLE repositories_new (
  id             TEXT    PRIMARY KEY,
  owner_id       TEXT    NOT NULL,
  name           TEXT    NOT NULL,
  visibility     TEXT    NOT NULL DEFAULT 'public',
  description    TEXT    NOT NULL DEFAULT '',
  default_branch TEXT    NOT NULL DEFAULT 'main',
  deleted_at     TEXT    NULL,
  created_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  updated_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  owner_type     TEXT    NOT NULL DEFAULT 'user'
    CHECK (owner_type IN ('user', 'org'))
);

INSERT INTO repositories_new (
  id, owner_id, name, visibility, description, default_branch,
  deleted_at, created_at, updated_at, owner_type
)
SELECT
  id, owner_id, name, visibility, description, default_branch,
  deleted_at, created_at, updated_at, 'user'
FROM repositories;

DROP TABLE repositories;
ALTER TABLE repositories_new RENAME TO repositories;

CREATE UNIQUE INDEX IF NOT EXISTS idx_repositories_owner_name_active
  ON repositories (owner_id, lower(name))
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_repositories_owner_id ON repositories(owner_id);

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS repository_collaborators (
  repo_id    TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  permission TEXT NOT NULL CHECK (permission IN ('read', 'write', 'admin')),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  PRIMARY KEY (repo_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_repository_collaborators_user_id
  ON repository_collaborators(user_id);
