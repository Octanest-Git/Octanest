-- logical: 0010_orgs_acl — organizations + ACL tables + polymorphic repository owner
-- Phase 09 owns 0009_ssh_keys; orgs use 0010 (D-ORG-01).
-- MySQL: TEXT cannot carry DEFAULT — use VARCHAR where a default is required.

CREATE TABLE IF NOT EXISTS organizations (
  id                       CHAR(36)     PRIMARY KEY,
  slug                     VARCHAR(39)  NOT NULL,
  display_name             VARCHAR(200) NOT NULL DEFAULT '',
  member_base_permission   VARCHAR(16)  NOT NULL DEFAULT 'none',
  created_at               TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at               TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  slug_lower               VARCHAR(39)  GENERATED ALWAYS AS (LOWER(slug)) STORED,
  UNIQUE KEY uk_organizations_slug (slug),
  UNIQUE KEY uk_organizations_slug_lower (slug_lower),
  CONSTRAINT organizations_member_base_check
    CHECK (member_base_permission IN ('none', 'read', 'write'))
) ENGINE=InnoDB;
CREATE TABLE IF NOT EXISTS organization_members (
  org_id     CHAR(36)  NOT NULL,
  user_id    CHAR(36)  NOT NULL,
  role       VARCHAR(16) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (org_id, user_id),
  CONSTRAINT fk_org_members_org FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE,
  CONSTRAINT fk_org_members_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT organization_members_role_check
    CHECK (role IN ('owner', 'admin', 'member'))
) ENGINE=InnoDB;

CREATE INDEX idx_organization_members_user_id ON organization_members(user_id);

CREATE TABLE IF NOT EXISTS organization_invites (
  id          CHAR(36)     PRIMARY KEY,
  org_id      CHAR(36)     NOT NULL,
  email       VARCHAR(320) NOT NULL,
  role        VARCHAR(16)  NOT NULL,
  token_hash  CHAR(64)     NOT NULL,
  expires_at  TIMESTAMP    NOT NULL,
  invited_by  CHAR(36)     NOT NULL,
  created_at  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  accepted_at TIMESTAMP    NULL,
  revoked_at  TIMESTAMP    NULL,
  UNIQUE KEY uk_organization_invites_token_hash (token_hash),
  CONSTRAINT fk_org_invites_org FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE,
  CONSTRAINT fk_org_invites_invited_by FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT organization_invites_role_check
    CHECK (role IN ('owner', 'admin', 'member'))
) ENGINE=InnoDB;

CREATE INDEX idx_organization_invites_org_id ON organization_invites(org_id);

CREATE TABLE IF NOT EXISTS repository_collaborators (
  repo_id    CHAR(36)    NOT NULL,
  user_id    CHAR(36)    NOT NULL,
  permission VARCHAR(16) NOT NULL,
  created_at TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (repo_id, user_id),
  CONSTRAINT fk_repo_collab_repo FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_repo_collab_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT repository_collaborators_permission_check
    CHECK (permission IN ('read', 'write', 'admin'))
) ENGINE=InnoDB;

CREATE INDEX idx_repository_collaborators_user_id ON repository_collaborators(user_id);

-- Polymorphic owner: drop user-only FK; backfill owner_type=user via DEFAULT.
ALTER TABLE repositories ADD COLUMN owner_type VARCHAR(16) NOT NULL DEFAULT 'user';
ALTER TABLE repositories DROP FOREIGN KEY fk_repositories_owner;
