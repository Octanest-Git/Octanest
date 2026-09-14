# Stack presets (community packs)

Octanest ships **in-repo stack presets** used by `/new` when creating a repository. There is **no public marketplace UI** yet — community presets are contributed as packs in this repository via pull request. A future marketplace can consume the same pack layout.

## Pack location

Presets live under:

```text
crates/octanest-api/assets/stack-presets/
```

- `catalog.json` — ordered list of packs with `id`, `label`, and `group` (drives the `/new` Stack template Select).
- `{pack-id}/` — directory of files copied into the initial commit when that pack is selected (README, scaffolding, etc.).

Related catalogs (also vendored; **not** fetched at request time):

| Catalog | Path |
|---------|------|
| `.gitignore` templates | `crates/octanest-api/assets/gitignore/` |
| Common SPDX license texts | `crates/octanest-api/assets/licenses/` |

The API resolves pack IDs through an allowlist from `catalog.json` only (no path traversal into arbitrary asset paths).

## Adding a community preset (PR)

1. Create `crates/octanest-api/assets/stack-presets/<pack-id>/` with the files that should appear in the initial commit.
2. Add an entry to `catalog.json` with a stable `id` (ascii letters, digits, hyphen/underscore), human `label`, and `group` (e.g. `Backend`, `Frontend`, `Systems`).
3. Keep packs small and self-explanatory — prefer a README plus minimal starter files over vendoring large frameworks.
4. Open a PR describing the stack and any license notes for included snippets.
5. After merge, rebuild the API so `include_dir` embeds the new assets.

## Create behavior

- If stack, license, and `.gitignore` are all **None** → bare empty repo (Quick setup / first-push guide).
- If any template content is selected → a single **Initial commit** on the default branch seeds those files.

## Marketplace (later)

A public browse/install marketplace UI is intentionally **out of scope** for now. Packs are designed so a later phase can index the same `catalog.json` + directory layout without rewriting contributor workflows.
