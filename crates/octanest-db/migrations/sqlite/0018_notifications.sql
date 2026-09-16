-- logical: 0018_notifications — in-app activity inbox (NOTF-01 / NOTF-02 / D-12)

CREATE TABLE IF NOT EXISTS notifications (
  id               TEXT PRIMARY KEY,
  recipient_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  actor_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  reason           TEXT NOT NULL,
  subject_kind     TEXT NOT NULL,
  subject_repo_id  TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  subject_number   INTEGER NOT NULL,
  subject_title    TEXT NOT NULL DEFAULT '',
  read_at          TEXT NULL,
  created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  CONSTRAINT notifications_subject_kind_check CHECK (subject_kind IN ('issue', 'pull_request'))
);

CREATE INDEX IF NOT EXISTS idx_notifications_recipient_created
  ON notifications (recipient_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_notifications_recipient_unread
  ON notifications (recipient_id, created_at DESC)
  WHERE read_at IS NULL;
