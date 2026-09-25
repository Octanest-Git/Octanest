-- logical: 0001_init — instances (D-09/D-10)
CREATE TABLE IF NOT EXISTS instances (
  id          INTEGER     PRIMARY KEY,
  dialect     TEXT        NOT NULL,
  probe_count BIGINT      NOT NULL DEFAULT 0,
  probed_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
