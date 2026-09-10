/** Shared origins for stack e2e (real API + Mailpit + stubs + OIDC mock). */

export function requireStack(): void {
  if (process.env.E2E_STACK !== "1") {
    throw new Error(
      "Stack e2e requires E2E_STACK=1 (use `make test-e2e-stack` / scripts/dev-auth/run-stack-e2e.sh)",
    );
  }
}

export const apiOrigin = () =>
  (process.env.OCTANEST_E2E_API_ORIGIN || "http://127.0.0.1:18080").replace(
    /\/$/,
    "",
  );

export const webOrigin = () =>
  (process.env.OCTANEST_E2E_WEB_ORIGIN || "http://127.0.0.1:13000").replace(
    /\/$/,
    "",
  );

export const mailpitOrigin = () =>
  (process.env.OCTANEST_E2E_MAILPIT_ORIGIN || "http://127.0.0.1:8025").replace(
    /\/$/,
    "",
  );

export const stubsOrigin = () =>
  (process.env.OCTANEST_E2E_STUBS_ORIGIN || "http://127.0.0.1:9092").replace(
    /\/$/,
    "",
  );

export const adminEmail = () =>
  process.env.OCTANEST_E2E_ADMIN_EMAIL || "admin@octanest.local";

export const adminPassword = () =>
  process.env.OCTANEST_E2E_ADMIN_PASSWORD || "password1";
