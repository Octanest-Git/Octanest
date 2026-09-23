import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectForgeRepoPackagesFlow } from "../lib/flows.ts";

describe("bun.webview PoC: forge repo packages", () => {
  beforeAll(() => {
    requireStack();
  });

  it("navigates to repo packages page", async () => {
    expect(await expectForgeRepoPackagesFlow()).toBe(true);
  }, 60_000);
});
