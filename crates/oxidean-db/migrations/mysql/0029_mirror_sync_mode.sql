-- logical: 0029_mirror_sync_mode — merge vs exact sync + LWW ref snapshot
-- MySQL: no IF NOT EXISTS on ADD COLUMN — idempotent via migrate runner / fresh DBs.
-- TEXT (not VARCHAR): utf8mb4 row limits cap VARCHAR at 16383.

ALTER TABLE repository_mirrors
  ADD COLUMN sync_mode VARCHAR(16) NOT NULL DEFAULT 'merge',
  ADD COLUMN last_ref_snapshot TEXT NOT NULL DEFAULT ('{}');
