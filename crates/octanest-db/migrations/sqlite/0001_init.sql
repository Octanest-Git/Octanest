-- logical: 0001_init — instances (D-09/D-10)
CREATE TABLE IF NOT EXISTS instances (
  id          INTEGER PRIMARY KEY,
  dialect     TEXT    NOT NULL,
  probe_count INTEGER NOT NULL DEFAULT 0,
  probed_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S','now'))
);
