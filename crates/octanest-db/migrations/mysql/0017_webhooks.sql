-- logical: 0017_webhooks — outbound repo webhooks + delivery history (HOOK-01..03)

CREATE TABLE IF NOT EXISTS webhooks (
  id             CHAR(36)      PRIMARY KEY,
  repository_id  CHAR(36)      NOT NULL,
  url            TEXT          NOT NULL,
  secret         TEXT          NOT NULL,
  active         TINYINT(1)    NOT NULL DEFAULT 1,
  events         TEXT          NOT NULL DEFAULT ('[]'),
  name           VARCHAR(255)  NOT NULL DEFAULT '',
  created_by     CHAR(36)      NOT NULL,
  created_at     TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at     TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_webhooks_repo FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE,
  CONSTRAINT fk_webhooks_created_by FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_webhooks_repository_id ON webhooks(repository_id);

CREATE TABLE IF NOT EXISTS webhook_deliveries (
  id              CHAR(36)      PRIMARY KEY,
  webhook_id      CHAR(36)      NOT NULL,
  delivery_guid   CHAR(36)      NOT NULL,
  event           VARCHAR(64)   NOT NULL,
  action          VARCHAR(64)   NOT NULL DEFAULT '',
  payload_json    MEDIUMTEXT    NOT NULL,
  status          VARCHAR(16)   NOT NULL DEFAULT 'pending',
  next_attempt_at TIMESTAMP     NULL,
  attempt_count   INT           NOT NULL DEFAULT 0,
  created_at      TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uk_webhook_deliveries_guid (delivery_guid),
  CONSTRAINT fk_webhook_deliveries_hook FOREIGN KEY (webhook_id) REFERENCES webhooks(id) ON DELETE CASCADE
);

CREATE INDEX idx_webhook_deliveries_webhook_id ON webhook_deliveries(webhook_id, created_at);
CREATE INDEX idx_webhook_deliveries_pending ON webhook_deliveries(status, next_attempt_at);

CREATE TABLE IF NOT EXISTS webhook_delivery_attempts (
  id               CHAR(36)      PRIMARY KEY,
  delivery_id      CHAR(36)      NOT NULL,
  attempt_number   INT           NOT NULL DEFAULT 1,
  attempted_at     TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  http_status      INT           NULL,
  error_message    TEXT          NULL,
  duration_ms      INT           NULL,
  response_snippet TEXT          NULL,
  CONSTRAINT fk_webhook_attempts_delivery FOREIGN KEY (delivery_id) REFERENCES webhook_deliveries(id) ON DELETE CASCADE
);

CREATE INDEX idx_webhook_delivery_attempts_delivery_id
  ON webhook_delivery_attempts(delivery_id, attempt_number);
