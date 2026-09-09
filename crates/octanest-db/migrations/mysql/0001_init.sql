-- logical: 0001_init — instances (D-09/D-10)
CREATE TABLE IF NOT EXISTS instances (
  id          INT         PRIMARY KEY,
  dialect     VARCHAR(16) NOT NULL,
  probe_count BIGINT      NOT NULL DEFAULT 0,
  probed_at   TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP
);
