-- logical: 0029_mirror_sync_mode — merge vs exact sync + LWW ref snapshot

ALTER TABLE repository_mirrors ADD COLUMN sync_mode TEXT NOT NULL DEFAULT 'merge';
ALTER TABLE repository_mirrors ADD COLUMN last_ref_snapshot TEXT NOT NULL DEFAULT '{}';
