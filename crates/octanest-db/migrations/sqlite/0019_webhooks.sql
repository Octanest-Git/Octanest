-- logical: 0019_webhooks — outbound repo webhooks + delivery history (HOOK-01..03)

CREATE TABLE IF NOT EXISTS webhooks (
  id             TEXT PRIMARY KEY,
  repository_id  TEXT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
  url            TEXT NOT NULL,
  secret         TEXT NOT NULL,
  active         INTEGER NOT NULL DEFAULT 1,
  events         TEXT NOT NULL DEFAULT '[]',
  name           TEXT NOT NULL DEFAULT '',
  created_by     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE INDEX IF NOT EXISTS idx_webhooks_repository_id ON webhooks(repository_id);

CREATE TABLE IF NOT EXISTS webhook_deliveries (
  id             TEXT PRIMARY KEY,
  webhook_id     TEXT NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
  delivery_guid  TEXT NOT NULL UNIQUE,
  event          TEXT NOT NULL,
  action         TEXT NOT NULL DEFAULT '',
  payload_json   TEXT NOT NULL,
  status         TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'success', 'failed')),
  next_attempt_at TEXT NULL,
  attempt_count  INTEGER NOT NULL DEFAULT 0,
  created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook_id
  ON webhook_deliveries(webhook_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_pending
  ON webhook_deliveries(status, next_attempt_at);

CREATE TABLE IF NOT EXISTS webhook_delivery_attempts (
  id               TEXT PRIMARY KEY,
  delivery_id      TEXT NOT NULL REFERENCES webhook_deliveries(id) ON DELETE CASCADE,
  attempt_number   INTEGER NOT NULL DEFAULT 1,
  attempted_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now')),
  http_status      INTEGER NULL,
  error_message    TEXT NULL,
  duration_ms      INTEGER NULL,
  response_snippet TEXT NULL
);

CREATE INDEX IF NOT EXISTS idx_webhook_delivery_attempts_delivery_id
  ON webhook_delivery_attempts(delivery_id, attempt_number);
