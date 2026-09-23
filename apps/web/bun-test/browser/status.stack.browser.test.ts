import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectStatusHealthy } from "../lib/flows.ts";

describe("bun.webview PoC: status", () => {
  beforeAll(() => {
    requireStack();
  });

  it("renders /status from system.health", async () => {
    expect(await expectStatusHealthy()).toBe(true);
  }, 45_000);
});
