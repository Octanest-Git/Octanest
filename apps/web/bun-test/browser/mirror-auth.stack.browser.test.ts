import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectMirrorAuthToggleFlow } from "../lib/flows.ts";

describe("bun.webview PoC: mirror auth toggle", () => {
  beforeAll(() => {
    requireStack();
  });

  it("clicks SSH deploy key without insertBefore / Octane pageerrors", async () => {
    expect(await expectMirrorAuthToggleFlow()).toBe(true);
  }, 90_000);
});
