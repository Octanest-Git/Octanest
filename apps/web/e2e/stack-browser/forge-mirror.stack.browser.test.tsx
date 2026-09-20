import { beforeAll, describe, expect, it } from "vitest";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectMirrorAuthToggleFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: repo mirror auth toggle", () => {
  beforeAll(() => {
    requireStack();
  });

  it("clicks SSH deploy key without insertBefore / Octane pageerrors", async () => {
    const ok = await commands.expectMirrorAuthToggleFlow();
    expect(ok).toBe(true);
  }, 90_000);
});
