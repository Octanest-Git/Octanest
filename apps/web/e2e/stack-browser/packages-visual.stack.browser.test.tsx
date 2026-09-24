import { beforeAll, describe, expect, it } from "vitest";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectPackagesVisualFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: packages visual baselines", () => {
  beforeAll(() => {
    requireStack();
  });

  it("matches committed screenshots for repo empty state and owner list", async () => {
    const ok = await commands.expectPackagesVisualFlow();
    expect(ok).toBe(true);
  }, 240_000);
});
