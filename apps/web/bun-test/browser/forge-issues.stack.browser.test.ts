import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectForgeIssuesCrudFlow } from "../lib/flows.ts";

describe("bun.webview PoC: forge issues", () => {
  beforeAll(() => {
    requireStack();
  });

  it("creates and closes issue via UI/RPC", async () => {
    expect(await expectForgeIssuesCrudFlow()).toBe(true);
  }, 90_000);
});
