import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectForgeSshAndOrgMembersFlow } from "../lib/flows.ts";

describe("bun.webview PoC: forge SSH and org members", () => {
  beforeAll(() => {
    requireStack();
  });

  it("renders SSH keys and org member pages", async () => {
    expect(await expectForgeSshAndOrgMembersFlow()).toBe(true);
  }, 90_000);
});
