# Watermarks-remover skills (vendored)

Upstream: [guillaumemeyer/watermarks-remover](https://github.com/guillaumemeyer/watermarks-remover)

Pinned: `v0.7.0` (`321d93d2efd6a8b26915c5eb5193d9d1701e2c4b`)

Skills:

- `remove-ai-marks/` — multi-vendor mark strip via HTTP service (`WATERMARKS_SERVICE_URL`, default `http://127.0.0.1:8765`)
- `clean-user-facing-text/` — self-contained prose hygiene (no service)

Refresh: clone the desired tag into `tmp/watermarks-remover`, copy `skills/*` here, update this pin, and refresh `.claude/skills/` symlinks if needed.

Do **not** vendor the Python `service/` into this repo. Start it from an upstream checkout (`make serve`) or Docker/GHCR per upstream README.

License: MIT — see [LICENSE-watermarks-remover](LICENSE-watermarks-remover) (copyright Guillaume Meyer and contributors).
