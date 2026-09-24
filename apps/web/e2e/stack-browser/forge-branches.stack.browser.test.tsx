import { beforeAll, describe, expect, it } from "vitest";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectBranchDialogsFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: branch dialogs (DOM race gate)", () => {
  beforeAll(() => {
    requireStack();
  });

  it("opens New branch + delete confirm without insertBefore races", async () => {
    const ok = await commands.expectBranchDialogsFlow();
    expect(ok).toBe(true);
  }, 90_000);
});
