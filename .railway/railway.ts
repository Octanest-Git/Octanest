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

export default defineRailway((ctx) => {
  const db = postgres("postgres");

  // Single forge volume covering Compose /var/* paths (D-CLOUD-04).
  // Subdirs: repos, lfs, packages, release-assets, uploads, ssh.
  const forgeData = volume("forge-data", { sizeMB: 20480 });

  // Wait for CI only on staging (autodeploy from main). Preview/production
  // keep GitHub connected but deployment triggers disabled in the dashboard.
  const stagingWaitForCi = ctx.isEnvironment("staging");

  const api = service("api", {
    source: github(REPO, { checkSuites: stagingWaitForCi || null }),
    build: {
      builder: "DOCKERFILE",
      dockerfilePath: "crates/octanest-api/Dockerfile",
      watchPatterns: [
        "crates/octanest-api/**",
        "crates/octanest-core/**",
        "crates/octanest-db/**",
        "crates/octanest-git/**",
        "packages/api-client/**",
      ],
    },
    healthcheck: "/health",
    // All environments migrate on API boot (including production promote). A
    // failed migration exits before listen so Railway keeps the previous replica.
    volumeMounts: {
      "/var": forgeData,
    },
    env: {
      DATABASE_URL: db.env.DATABASE_URL,
      // Set per environment in the dashboard (preview / staging / production).
      OCTANEST_ENV: preserve(),
      OCTANEST_DB_DIALECT: "postgres",
      OCTANEST_AUTO_MIGRATE: "true",
      OCTANEST_ALLOW_SIGNUP: "true",
      // Branch-protection update-hook helper (same path as Compose / API image).
      OCTANEST_PROTECTION_HELPER: "/usr/local/bin/octanest-protection-hook",
      // Align Railway healthcheck PORT with the API listen address.
      PORT: "8080",
      API_BIND: "0.0.0.0:8080",
      OCTANEST_REPOS_DIR: "/var/repos",
      OCTANEST_LFS_DIR: "/var/lfs",
      OCTANEST_PACKAGES_DIR: "/var/packages",
      OCTANEST_RELEASE_ASSETS_DIR: "/var/release-assets",
      OCTANEST_SSH_HOST_KEY_DIR: "/var/ssh",
      OCTANEST_SSH_ENABLED: "true",
      // Default SSH port (clients expect 22). Railway still publishes a random
      // public TCP proxy port → this application port.
      OCTANEST_SSH_PORT: "22",
      // Track gateway public domain (PR Environments get unique hosts; do not preserve).
      OCTANEST_PUBLIC_ORIGIN: "https://${{gateway.RAILWAY_PUBLIC_DOMAIN}}",
      OCTANEST_CORS_ORIGINS: "https://${{gateway.RAILWAY_PUBLIC_DOMAIN}}",
      OCTANEST_ADMIN_EMAIL: preserve(),
      OCTANEST_ADMIN_PASSWORD: preserve(),
      OCTANEST_RESEND_API_KEY: preserve(),
      OCTANEST_SMTP_URL: preserve(),
      OCTANEST_MAIL_FROM: preserve(),
      // AES-256-GCM for Actions secrets, mirror credentials, webhook secrets (D-ACT-17).
      // Required — set a unique value per environment in the dashboard (never commit).
      OCTANEST_ACTIONS_SECRETS_KEY: preserve(),
      OCTANEST_SSH_HOST: "${{gateway.RAILWAY_PUBLIC_DOMAIN}}",
      WORKOS_API_KEY: preserve(),
      WORKOS_CLIENT_ID: preserve(),
      OCTANEST_OIDC_ISSUER: preserve(),
      OCTANEST_OIDC_CLIENT_ID: preserve(),
      OCTANEST_OIDC_CLIENT_SECRET: preserve(),
    },
    // Git-over-SSH TCP publish (D-CLOUD-08).
    tcp: [22],
  });

  const web = service("web", {
    source: github(REPO, { checkSuites: stagingWaitForCi || null }),
    build: {
      builder: "DOCKERFILE",
      dockerfilePath: "apps/web/Dockerfile",
      watchPatterns: ["apps/web/**", "packages/**"],
    },
    // Private behind gateway; SPA `/` can 403 for bare probes. Rely on gateway + api checks.
    env: {
      // Pin listen port so gateway WEB_PORT=3000 stays correct on private net.
      PORT: "3000",
      // SSR / server-fn RPCs must hit the API on the private network (not
      // 127.0.0.1:8080). Browser clients still use same-origin via gateway.
      OCTANEST_API_ORIGIN: "http://${{api.RAILWAY_PRIVATE_DOMAIN}}:8080",
      // Public site origin — track gateway domain (PR Environments need this).
      OCTANEST_PUBLIC_ORIGIN: "https://${{gateway.RAILWAY_PUBLIC_DOMAIN}}",
      // Vite preview Host allowlist (comma-separated; leading `.` = suffix).
      // Example: `.up.railway.app,octanest.jereko.dev` — see docs/CONFIGURATION.md.
      OCTANEST_VITE_ALLOWED_HOSTS: preserve(),
      // CloneBox SSH advertise (SSR’d into the page; match api listen/advertise).
      OCTANEST_SSH_HOST: "${{gateway.RAILWAY_PUBLIC_DOMAIN}}",
      OCTANEST_SSH_PORT: "22",
    },
  });

  const gateway = service("gateway", {
    source: github(REPO, { checkSuites: stagingWaitForCi || null }),
    build: {
      builder: "DOCKERFILE",
      dockerfilePath: "deploy/cloud/Dockerfile",
      watchPatterns: ["deploy/cloud/**"],
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
