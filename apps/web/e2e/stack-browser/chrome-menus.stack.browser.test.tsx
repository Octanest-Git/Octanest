import { beforeAll, describe, expect, it } from "@octanest/web/test-runner";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectChromeCreateAndAccountMenusFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: chrome Create + Account menus", () => {
  beforeAll(() => {
    requireStack();
  });

  it("hides create/account menus when anonymous and shows them when signed in", async () => {
    const ok = await commands.expectChromeCreateAndAccountMenusFlow();
    expect(ok).toBe(true);
  }, 180_000);
});
