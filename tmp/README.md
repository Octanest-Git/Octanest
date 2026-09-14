# Temporary scratch (`tmp/`)

Local and agent scratch space. **Do not commit contents** — everything under this directory except this README is gitignored.

## Use for

- Screenshots, HAR dumps, and Playwright traces
- One-off scripts, patches, and export dumps
- Debug logs and intermediate build artifacts
- Any file that should not land in the repo root or source trees

## Do not use for

- Source of truth (code, migrations, planning docs)
- Secrets or credentials
- Files that need to ship with the product

## Agents

Put temporary files here as `tmp/<purpose>-<short-id>/…` or `tmp/<filename>`. Prefer deleting when done. Never write scratch files to the repository root.
