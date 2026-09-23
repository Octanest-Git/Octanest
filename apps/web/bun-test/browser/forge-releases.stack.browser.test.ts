import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectForgeReleasesCrudFlow } from "../lib/flows.ts";

describe("bun.webview PoC: forge releases", () => {
  beforeAll(() => {
    requireStack();
  });

  it("creates release via UI/RPC", async () => {
    expect(await expectForgeReleasesCrudFlow()).toBe(true);
  }, 90_000);
});
