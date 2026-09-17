# Stack presets

Octanest ships **in-repo stack presets** used by `/new` when creating a repository. Built-in packs are embedded into the API binary at compile time (`include_dir`) — they are **not** fetched from the internet at create time.

Instance admins can also upload custom packs (see Admin → Templates). Users can mark a repository as a template and create new repos from its default-branch tree. A public marketplace UI remains out of scope.

## Pack location

```text
crates/octanest-api/assets/stack-presets/
```

- `catalog.json` — ordered list of packs. Required fields: `id`, `label`, `group`, `description`, `source`, `source_ref`, `last_synced`. Optional: `default_gitignore`, `verify_commands`.
- `{pack-id}/` — files copied into the initial commit when that pack is selected.

Related catalogs (also vendored):

| Catalog | Path |
|---------|------|
| `.gitignore` templates | `crates/octanest-api/assets/gitignore/` |
| Common SPDX license texts | `crates/octanest-api/assets/licenses/` |

Pack IDs are allowlisted from `catalog.json` only (no path traversal).

### Catalog `source` values

| Value | Meaning |
|-------|---------|
| `cli` | Generated from an official create/init CLI (`source_ref` holds the command) |
| `repo` | Snapshot of an upstream template repo/tag |
| `manual` | Hand-maintained (e.g. `empty`) |
| `octanest` | First-party Octanest starters (Octane, Ripple) |

## Refresh workflow (built-ins)

1. Read `source_ref` for the pack in `catalog.json`.
2. Run the CLI (or checkout the pinned tag) into a **temp** directory with non-interactive flags.
3. Strip noise: `.git/`, `node_modules/`, `.env*`, OS junk, toolchain caches.
4. Copy the resulting tree into `stack-presets/<id>/` (replace files).
5. Ensure a `README.md` documents the happy path (`npm run dev`, `cargo run`, …).
6. Stamp metadata: `bun scripts/sync-stack-presets.mjs --pack <id> --stamp`
7. Validate: `make check-stack-presets`
8. Rebuild the API so `include_dir` picks up the new assets.

Do **not** vendor `node_modules` or SDK binaries. Lockfiles are allowed when the pack stays under the size budget.

### Size budget

| Limit | Value |
|-------|-------|
| Per pack | ≤ 2 MiB of tracked files |
| All packs | ≤ 40 MiB total under `stack-presets/` |

Enforced by `make check-stack-presets` / `scripts/check-stack-presets.sh`.

## Adding or updating a built-in pack

1. Create or refresh `crates/octanest-api/assets/stack-presets/<pack-id>/`.
2. Add or update the `catalog.json` entry (`source` / `source_ref` / `last_synced` required).
3. Prefer official-style starters that are runnable after install — not one-file Hello World stubs.
4. Open a PR describing the stack and license notes for included snippets.
5. After merge, rebuild the API.

## Create behavior

- If stack, instance pack, template repo, license, and `.gitignore` are all none → bare empty repo.
- If any template content is selected → a single **Initial commit** on the default branch seeds those files.
- `stack_id`, `instance_pack_id`, and `template_repo_id` are mutually exclusive.

## Instance + user templates

- **Instance packs:** sys-admins upload a zip under Admin → Templates. Enabled packs appear on `/new` with provenance `instance`.
- **User templates:** repo admins toggle “Template repository” in settings. Others who can **read** the repo can create from its default-branch tip (fresh initial commit — not a fork).

## Marketplace (later)

A public browse/install marketplace UI is intentionally **out of scope**. Pack layout and catalog metadata are designed so a later phase can index the same structure without rewriting contributor workflows.
