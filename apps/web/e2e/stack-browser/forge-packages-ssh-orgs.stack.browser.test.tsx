import { beforeAll, describe, expect, it } from "vitest";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectForgeSshAndOrgMembersFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: SSH keys + org members (D-QH-03)", () => {
  beforeAll(() => {
    requireStack();
  });

  it("adds an SSH key and opens org members", async () => {
    const ok = await commands.expectForgeSshAndOrgMembersFlow();
    expect(ok).toBe(true);
  }, 120_000);
});
