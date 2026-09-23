/** Env helpers for bun:test PoC (mirrors e2e/stack/env.ts). */

function envGet(key: string): string | undefined {
  const v = process.env[key];
  if (v !== undefined && v !== "") return v;
  return undefined;
}

export function requireStack(): void {
  if (envGet("E2E_STACK") !== "1") {
    throw new Error(
      "Stack e2e requires E2E_STACK=1 (use `make test-bun-poc-browser` / run-stack-e2e with BUN_TEST_POC=1)",
    );
  }
}

export const apiOrigin = () =>
  (envGet("OCTANEST_E2E_API_ORIGIN") || "http://127.0.0.1:18080").replace(/\/$/, "");

export const webOrigin = () =>
  (envGet("OCTANEST_E2E_WEB_ORIGIN") || "http://127.0.0.1:13000").replace(/\/$/, "");

export const adminEmail = () => envGet("OCTANEST_E2E_ADMIN_EMAIL") || "admin@octanest.local";

export const adminPassword = () => envGet("OCTANEST_E2E_ADMIN_PASSWORD") || "password1";

export const e2eDbPath = () => envGet("OCTANEST_E2E_DB_PATH");
