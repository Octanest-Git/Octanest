import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectChromeCreateAndAccountMenusFlow } from "../lib/flows.ts";

describe("bun.webview PoC: chrome Create + Account menus", () => {
  beforeAll(() => {
    requireStack();
  });

  it("hides create/account menus when anonymous and shows them when signed in", async () => {
    expect(await expectChromeCreateAndAccountMenusFlow()).toBe(true);
  }, 180_000);
});
