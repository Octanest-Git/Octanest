/** Shared origins for stack e2e (real API + Mailpit + stubs + OIDC mock). */

/**
 * Browser-safe env read. Vitest browser has no Node `process` unless Vite `define`
 * replaces static `process.env.KEY` accesses — prefer optional chaining either way.
 */
function envGet(key: string): string | undefined {
  try {
    // Bracket access so Vite define cannot rewrite static process.env.KEY.
    const proc = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process;
    const v = proc?.env?.[key];
    if (v !== undefined && v !== "") return v;
  } catch {
    // ignore
  }
  return undefined;
}

export function requireStack(): void {
  if (envGet("E2E_STACK") !== "1") {
    throw new Error(
      "Stack e2e requires E2E_STACK=1 (use `make test-e2e-stack` / scripts/dev-auth/run-stack-e2e.sh)",
    );
  }
}

export const apiOrigin = () =>
  (envGet("OCTANEST_E2E_API_ORIGIN") || "http://127.0.0.1:18080").replace(/\/$/, "");

export const webOrigin = () =>
  (envGet("OCTANEST_E2E_WEB_ORIGIN") || "http://127.0.0.1:13000").replace(/\/$/, "");

export const mailpitOrigin = () =>
  (envGet("OCTANEST_E2E_MAILPIT_ORIGIN") || "http://127.0.0.1:8025").replace(/\/$/, "");

export const stubsOrigin = () =>
  (envGet("OCTANEST_E2E_STUBS_ORIGIN") || "http://127.0.0.1:9092").replace(/\/$/, "");

export const adminEmail = () => envGet("OCTANEST_E2E_ADMIN_EMAIL") || "admin@octanest.local";

export const adminPassword = () => envGet("OCTANEST_E2E_ADMIN_PASSWORD") || "password1";

export const e2eDbPath = () => envGet("OCTANEST_E2E_DB_PATH");
