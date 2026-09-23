import { beforeAll, describe, expect, it } from "@octanest/web/test-runner";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectForgeRepoPackagesFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: forge repo home + packages (D-QH-03)", () => {
  beforeAll(() => {
    requireStack();
  });

  it("opens seeded repo chrome and packages list", async () => {
    const ok = await commands.expectForgeRepoPackagesFlow();
    expect(ok).toBe(true);
  }, 60_000);
});
