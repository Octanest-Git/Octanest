import { beforeAll, describe, expect, it } from "vitest";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectAdminLfsQuotasFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: forge admin LFS + packages quotas (G-11.1-15)", () => {
  beforeAll(() => {
    requireStack();
  });

  it("opens /admin/lfs (and packages) as forge admin without error overlay", async () => {
    const ok = await commands.expectAdminLfsQuotasFlow();
    expect(ok).toBe(true);
  }, 120_000);
});
