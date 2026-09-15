import { beforeAll } from "vitest";
import { webOrigin } from "../stack/env";

beforeAll(async () => {
  // requireStack() lives in env.ts and is browser-safe (no bare `process`).
  const { requireStack } = await import("../stack/env");
  requireStack();

  const web = webOrigin();
  const res = await fetch(web).catch(() => null);
  if (!res?.ok) {
    throw new Error(`E2E_STACK=1 but web origin failed at ${web} — run make test-e2e-stack`);
  }
});
