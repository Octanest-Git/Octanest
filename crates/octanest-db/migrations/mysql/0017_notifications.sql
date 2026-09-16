-- logical: 0017_notifications — in-app activity inbox (NOTF-01 / NOTF-02 / D-12)

CREATE TABLE IF NOT EXISTS notifications (
  id               CHAR(36)      PRIMARY KEY,
  recipient_id     CHAR(36)      NOT NULL,
  actor_id         CHAR(36)      NOT NULL,
  reason           VARCHAR(64)   NOT NULL,
  subject_kind     VARCHAR(32)   NOT NULL,
  subject_repo_id  CHAR(36)      NOT NULL,
  subject_number   BIGINT        NOT NULL,
  subject_title    VARCHAR(500)  NOT NULL DEFAULT '',
  read_at          TIMESTAMP     NULL,
  created_at       TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_notifications_recipient FOREIGN KEY (recipient_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_notifications_actor FOREIGN KEY (actor_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_notifications_repo FOREIGN KEY (subject_repo_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT notifications_subject_kind_check CHECK (subject_kind IN ('issue', 'pull_request'))
) ENGINE=InnoDB;

CREATE INDEX idx_notifications_recipient_created
  ON notifications (recipient_id, created_at);
CREATE INDEX idx_notifications_recipient_unread
  ON notifications (recipient_id, read_at, created_at);
