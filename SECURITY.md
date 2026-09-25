# Security policy

Oxidean is a self-hostable forge: it stores source code, credentials, and user data, and it exposes git over SSH and HTTP, LFS, and package registries. Reports are taken seriously.

## Supported versions

Security fixes land on `main` and the latest release. Older tags are not patched, so please upgrade before reporting a bug against an old version.

## Reporting a vulnerability

Do not file public issues, pull requests, or discussions for vulnerabilities.

Use [GitHub private vulnerability reporting](https://github.com/oxidean/oxidean/security/advisories/new) on `oxidean/oxidean`.

A good report includes:

- the version, image tag, or commit you tested
- how the instance runs (Oxidean Cloud, Compose, or other self-hosted)
- steps to reproduce and the impact you see
- whether the issue needs specific configuration or permissions to trigger

Please give us reasonable time to investigate and release a fix before disclosing publicly. We will acknowledge reports as soon as we can; this is an early-stage project, so there is no formal SLA.

## Scope

In scope: the Oxidean codebase itself — the API, web UI, git/SSH/LFS surfaces, auth and session handling, organizations and permissions, mirroring, actions runners, and package registry.

Out of scope:

- weaknesses of a specific self-hosted deployment (open ports, weak `OXIDEAN_*` settings, unpatched host)
- issues in third-party dependencies without a demonstrated exploit path through Oxidean
- denial-of-service via resource exhaustion that applies to any unauthenticated web service

When in doubt, report privately anyway and let us triage.
