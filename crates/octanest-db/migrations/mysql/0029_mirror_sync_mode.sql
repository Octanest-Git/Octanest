-- logical: 0029_mirror_sync_mode — merge vs exact sync + LWW ref snapshot
-- MySQL: no IF NOT EXISTS on ADD COLUMN — idempotent via migrate runner / fresh DBs.

ALTER TABLE repository_mirrors
  ADD COLUMN sync_mode VARCHAR(16) NOT NULL DEFAULT 'merge',
  ADD COLUMN last_ref_snapshot VARCHAR(16384) NOT NULL DEFAULT '{}';
