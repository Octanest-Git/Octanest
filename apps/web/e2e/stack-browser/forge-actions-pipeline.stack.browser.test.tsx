import { beforeAll, describe, expect, it } from "vitest";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectActionsPipelineFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: actions pipeline (runner → green run)", () => {
  beforeAll(() => {
    requireStack();
  });

  it("push-triggered workflow runs green on the bundled runner and streams logs", async () => {
    const ok = await commands.expectActionsPipelineFlow();
    expect(ok).toBe(true);
  }, 240_000);
});
