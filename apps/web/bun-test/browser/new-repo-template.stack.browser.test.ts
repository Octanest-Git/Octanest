import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectNewRepoTemplatePickerFlow } from "../lib/flows.ts";

describe("bun.webview PoC: /new template picker", () => {
  beforeAll(() => {
    requireStack();
  });

  it("picks a stack template without Octane DOM-race pageerrors", async () => {
    expect(await expectNewRepoTemplatePickerFlow()).toBe(true);
  }, 180_000);
});
