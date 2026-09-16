/**
 * Octanest Cloud — Railway TypeScript IaC (D-CLOUD-01…07).
 *
 * Same Dockerfiles as Compose. Managed Postgres. File-configured Caddy gateway
 * (no Docker socket). Secrets use preserve() — never commit production tokens.
 *
 * Operators: `railway config plan` then apply only with explicit approval.
 * See .railway/README.md and docs/DEPLOYMENT.md.
 */
import {
  defineRailway,
  github,
  group,
  postgres,
  preserve,
  project,
  service,
  volume,
} from "railway/iac";

/** GitHub source for DOCKERFILE builds (repo-root context). */
const REPO = "Octanest-Git/Octanest";

export default defineRailway(() => {
  const db = postgres("postgres");

  // Single forge volume covering Compose /var/* paths (D-CLOUD-04).
  // Subdirs: repos, lfs, packages, release-assets, uploads, ssh.
  const forgeData = volume("forge-data", { sizeMB: 20480 });

  const api = service("api", {
    source: github(REPO),
    build: {
      builder: "DOCKERFILE",
      dockerfilePath: "crates/octanest-api/Dockerfile",
    },
    healthcheck: "/health",
    // API image has no migrate binary; keep AUTO_MIGRATE=false and run migrations
    // via a one-off / temporary AUTO_MIGRATE=true first boot (see docs/DEPLOYMENT.md).
    volumeMounts: {
      "/var": forgeData,
    },
    env: {
      DATABASE_URL: db.env.DATABASE_URL,
      OCTANEST_ENV: "production",
      OCTANEST_DB_DIALECT: "postgres",
      OCTANEST_AUTO_MIGRATE: "false",
      OCTANEST_ALLOW_SIGNUP: "true",
      OCTANEST_REPOS_DIR: "/var/repos",
      OCTANEST_LFS_DIR: "/var/lfs",
      OCTANEST_PACKAGES_DIR: "/var/packages",
      OCTANEST_RELEASE_ASSETS_DIR: "/var/release-assets",
      OCTANEST_SSH_HOST_KEY_DIR: "/var/ssh",
      OCTANEST_SSH_ENABLED: "true",
      OCTANEST_SSH_PORT: "2222",
      // Public origin + CORS — set in dashboard / preserve existing (D-CLOUD-06).
      OCTANEST_PUBLIC_ORIGIN: preserve(),
      OCTANEST_CORS_ORIGINS: preserve(),
      OCTANEST_ADMIN_EMAIL: preserve(),
      OCTANEST_ADMIN_PASSWORD: preserve(),
      OCTANEST_RESEND_API_KEY: preserve(),
      OCTANEST_SMTP_URL: preserve(),
      WORKOS_API_KEY: preserve(),
      WORKOS_CLIENT_ID: preserve(),
      OCTANEST_OIDC_ISSUER: preserve(),
      OCTANEST_OIDC_CLIENT_ID: preserve(),
      OCTANEST_OIDC_CLIENT_SECRET: preserve(),
    },
    // Optional Git-over-SSH TCP publish when the platform supports it (D-CLOUD-08).
    tcp: [2222],
  });

  const web = service("web", {
    source: github(REPO),
    build: {
      builder: "DOCKERFILE",
      dockerfilePath: "apps/web/Dockerfile",
    },
    healthcheck: "/",
    env: {
      // Web talks to API via public origin / same-origin through gateway.
      OCTANEST_PUBLIC_ORIGIN: preserve(),
    },
  });

  const gateway = service("gateway", {
    source: github(REPO),
    build: {
      builder: "DOCKERFILE",
      dockerfilePath: "deploy/cloud/Dockerfile",
    },
    // Public HTTP(S) edge — attach custom domain in Railway dashboard.
    env: {
      API_HOST: api.env.RAILWAY_PRIVATE_DOMAIN,
      API_PORT: "8080",
      WEB_HOST: web.env.RAILWAY_PRIVATE_DOMAIN,
      WEB_PORT: "3000",
    },
  });

  const forge = group("Octanest Cloud", [db, forgeData, api, web, gateway]);

  return project("octanest-cloud", {
    resources: [forge],
  });
});
