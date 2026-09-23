import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { signupThroughUi } from "../lib/flows.ts";
import { updateAuthSettings, adminLogin } from "../../e2e/stack/client.ts";

describe("bun.webview PoC: signup", () => {
  beforeAll(async () => {
    requireStack();
    const cookie = await adminLogin();
    await updateAuthSettings(cookie, {
      provider_mode: "local",
      email_provider: "log",
    });
  });

  it("signs up through the web UI", async () => {
    const suffix = Date.now();
    expect(
      await signupThroughUi({
        email: `bun.ui.${suffix}@octanest.local`,
        username: `bunui${suffix}`,
        password: "password1",
      }),
    ).toBe(true);
  }, 120_000);
});
