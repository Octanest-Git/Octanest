# Octanest

<p align="center">
  <img src="brand/octanest-mark.png" alt="Octanest" width="128" height="128" />
</p>

<p align="center">
  <a href="https://github.com/Octanest-Git/Octanest/actions/workflows/ci.yml"><img src="https://github.com/Octanest-Git/Octanest/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" /></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/version-0.1.0-informational.svg" alt="Version 0.1.0" /></a>
</p>

## What it is

A self-hostable GitHub-style forge — git hosting, issues, organizations, and package registries — that runs as **one product** for Octanest Cloud and on your own machines.

## Who it’s for

- **Operators** who want a Compose-based forge they control
- **Teams** that need a GitHub-shaped workflow without splitting cloud vs self-host into different products
- **Contributors** improving the same codebase that powers both deployments

## Cloud vs self-host

**Octanest Cloud** and **self-hosted Octanest** share the same images and application. Self-host with Docker Compose today; Cloud is the hosted instance of that same stack. See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

## Quick start

Prerequisites: Docker Engine with Compose v2, and a free host port **80** (Traefik). Git-over-SSH also publishes host port **2222** by default.

```bash
cp .env.example .env
make up
make smoke
```

Open [http://localhost](http://localhost). Tear down with `make down`.

Full walkthrough (including host-side development): [docs/GETTING-STARTED.md](docs/GETTING-STARTED.md).  
Production-oriented Compose notes: [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

## What’s included

Shipped on the current mainline:

- Git hosting over **HTTPS** (Smart HTTP + PATs) and **SSH**
- **Organizations**, collaborators, and repository visibility / ACL
- **Issues** (comments, labels, assignees)
- **Git LFS**, **releases** with assets, repo rename / transfer
- **Package registries** (OCI, npm, generic/raw)
- Multi-database support: **Postgres**, **MySQL**, **SQLite**

Coming later (not shipped yet): full pull-request review/merge, branch protection, search, notifications, webhooks, Actions, and social explore.

## Docs

Canonical docs live under [`docs/`](docs/).

**Run & operate**

- [`docs/GETTING-STARTED.md`](docs/GETTING-STARTED.md) — first run (Compose or host)
- [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) — Compose images, Traefik, volumes, production knobs
- [`docs/CONFIGURATION.md`](docs/CONFIGURATION.md) — environment variables
- [`docs/database.md`](docs/database.md) — Postgres / MySQL / SQLite, migrations, dialect switching

**Product & API**

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — system overview (git, SSH, LFS, orgs, packages)
- [`docs/API.md`](docs/API.md) — RPC + Smart HTTP / LFS / SSH / registry surfaces
- [`docs/guides/stack-presets.md`](docs/guides/stack-presets.md) — in-repo `/new` stack presets

**Develop**

- [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) — local API + Vite, RPC sync, monorepo layout
- [`docs/TESTING.md`](docs/TESTING.md) — nextest + Vitest + stack e2e
- [`docs/dev-auth.md`](docs/dev-auth.md) — auth/email stubs without cloud secrets
- [`docs/CODE_PRACTICES.md`](docs/CODE_PRACTICES.md) — conventions for humans and agents

## Contributing

Want to change the code? Start with [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/CODE_PRACTICES.md](docs/CODE_PRACTICES.md). Agents: [AGENTS.md](AGENTS.md) (Octane ≠ React).

## License

[MIT](LICENSE) © Octanest contributors
