import { beforeAll } from "vitest";

beforeAll(async () => {
  if (process.env.E2E_STACK !== "1") {
    // Soft-skip the whole file when stack harness is not running.
    // Vitest still loads the project; individual tests call requireStack().
    return;
  }
  const api = (process.env.OCTANEST_E2E_API_ORIGIN || "http://127.0.0.1:18080").replace(/\/$/, "");
  const health = await fetch(`${api}/health`).catch(() => null);
  if (!health?.ok) {
    throw new Error(`E2E_STACK=1 but API health failed at ${api}/health — run make test-e2e-stack`);
  }
});
