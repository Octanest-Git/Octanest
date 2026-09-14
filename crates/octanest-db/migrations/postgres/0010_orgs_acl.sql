-- logical: 0010_orgs_acl — organizations + ACL tables + polymorphic repository owner
-- Phase 09 owns 0009_ssh_keys; orgs use 0010 (D-ORG-01).

CREATE TABLE IF NOT EXISTS organizations (
  id                       TEXT        PRIMARY KEY,
  slug                     VARCHAR(39) NOT NULL UNIQUE,
  display_name             TEXT        NOT NULL DEFAULT '',
  member_base_permission   TEXT        NOT NULL DEFAULT 'none',
  created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT organizations_member_base_check
    CHECK (member_base_permission IN ('none', 'read', 'write'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_organizations_slug_lower
  ON organizations (lower(slug));

CREATE TABLE IF NOT EXISTS organization_members (
  org_id     TEXT        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  user_id    TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role       TEXT        NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (org_id, user_id),
  CONSTRAINT organization_members_role_check
    CHECK (role IN ('owner', 'admin', 'member'))
);

CREATE INDEX IF NOT EXISTS idx_organization_members_user_id
  ON organization_members(user_id);

CREATE TABLE IF NOT EXISTS organization_invites (
  id          TEXT        PRIMARY KEY,
  org_id      TEXT        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  email       TEXT        NOT NULL,
  role        TEXT        NOT NULL,
  token_hash  CHAR(64)    NOT NULL UNIQUE,
  expires_at  TIMESTAMPTZ NOT NULL,
  invited_by  TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  accepted_at TIMESTAMPTZ NULL,
  revoked_at  TIMESTAMPTZ NULL,
  CONSTRAINT organization_invites_role_check
    CHECK (role IN ('owner', 'admin', 'member'))
);

CREATE INDEX IF NOT EXISTS idx_organization_invites_org_id
  ON organization_invites(org_id);

CREATE TABLE IF NOT EXISTS repository_collaborators (
  repo_id    TEXT        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  user_id    TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  permission TEXT        NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (repo_id, user_id),
  CONSTRAINT repository_collaborators_permission_check
    CHECK (permission IN ('read', 'write', 'admin'))
);

CREATE INDEX IF NOT EXISTS idx_repository_collaborators_user_id
  ON repository_collaborators(user_id);

-- Polymorphic owner: keep owner_id as TEXT; drop user-only FK (backfill owner_type=user).
ALTER TABLE repositories ADD COLUMN owner_type TEXT NOT NULL DEFAULT 'user';
ALTER TABLE repositories ADD CONSTRAINT repositories_owner_type_check
  CHECK (owner_type IN ('user', 'org'));
ALTER TABLE repositories DROP CONSTRAINT IF EXISTS repositories_owner_id_fkey;
