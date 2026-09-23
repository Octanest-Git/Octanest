import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectAdminLfsQuotasFlow } from "../lib/flows.ts";

describe("bun.webview PoC: forge admin", () => {
  beforeAll(() => {
    requireStack();
  });

  it("renders admin LFS, packages, and auth pages", async () => {
    expect(await expectAdminLfsQuotasFlow()).toBe(true);
  }, 90_000);
});
