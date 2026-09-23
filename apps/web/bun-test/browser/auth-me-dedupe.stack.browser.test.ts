import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectAuthMeDedupedOnHome } from "../lib/flows.ts";
import { adminLogin, updateAuthSettings } from "../../e2e/stack/client.ts";

describe("bun.webview PoC: auth.me Query dedupe", () => {
  beforeAll(async () => {
    requireStack();
    const cookie = await adminLogin();
    await updateAuthSettings(cookie, {
      provider_mode: "local",
      email_provider: "log",
    });
  });

  it("dedupes auth.me on signed-in home (chrome + banner share cache)", async () => {
    expect(await expectAuthMeDedupedOnHome()).toBe(true);
  }, 90_000);
});
