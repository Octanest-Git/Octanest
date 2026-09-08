# Octanest

## What This Is

Octanest is a GitHub competitor: a social coding platform for hosting git repositories, collaborating on code (pull requests, issues, reviews), and shipping software together. The same product runs two ways — as a public cloud (Octanest Cloud) and as self-hosted software anyone can run on their own infrastructure.

## Core Value

One forge you can trust in the cloud or on your own machines — without splitting into separate “hosted brand” vs “self-host software” products.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

(None yet — ship to validate)

### Active

<!-- Current scope. Building toward these. -->

- [ ] Git hosting over SSH and HTTPS (clone, push, pull, browse)
- [ ] Users, organizations, and repository permissions
- [ ] Pull requests with review and merge
- [ ] Issues and basic project collaboration
- [ ] Public Octanest Cloud deployment of the same codebase
- [ ] Self-host install path (same binary/image as cloud)

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- Replacing git with a custom VCS — stay git-compatible so existing workflows work
- Forking Gitea/Forgejo as the product identity — Octanest is its own brand and codebase direction
- Separate cloud-only vs self-host-only feature forks — dual-mode means one product

## Context

- **Brand:** Octanest = octane (performance / octa wink) + nest (where repos live). Chosen over Octabase (collides with AFFiNE’s OctoBase) and Octahub (too GitHub-formula + OctoHub collisions).
- **Model:** GitLab-style dual-mode (one product, cloud + self-host), not Codeberg/Forgejo split (hosted instance vs different software brand).
- **Stack direction:** TypeScript workspace under `personal/typescript`; OctaneJS is already in the wider toolchain for other projects — do not name this product bare “Octane.”
- **Name availability (2026-09-08, informational):**
  - npm `octanest`: free
  - crates.io `octanest`: free
  - Docker Hub `library/octanest`: not found (free to claim)
  - GitHub org [`octanest`](https://github.com/octanest): **already exists** (1 public repo) — may need a different org slug or outreach
  - `octanest.com`: DNS resolves (consumer/LLP brand elsewhere)
  - `octanest.dev` / `.io` / `.app`: no DNS at check time (candidates)

## Constraints

- **Brand:** Keep the octa/octane connection; never ship as “GitHub clone” branding
- **Distribution:** Cloud and self-host must share one codebase and release train
- **Compatibility:** Real git clients and remotes must work (`git@…:user/repo.git`)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Product name: **Octanest** | Octane DNA + nest metaphor; softer collisions than Octabase/Octahub | — Pending |
| Dual-mode (cloud + self-host) | Compete with GitHub while letting people run their own instance | — Pending |
| One brand for both modes | Avoid Codeberg/Forgejo-style brand split | — Pending |

---
*Last updated: 2026-09-08 after naming lock and repo scaffold*
