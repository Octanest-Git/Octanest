import { beforeAll, describe, expect, it } from "@octanest/web/test-runner";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectForgeSshAndOrgMembersFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: SSH keys + org settings (members/general/labels)", () => {
  beforeAll(() => {
    requireStack();
  });

  it("adds an SSH key and covers org settings sidebar pages (happy)", async () => {
    const ok = await commands.expectForgeSshAndOrgMembersFlow();
    expect(ok).toBe(true);
  }, 240_000);
});
