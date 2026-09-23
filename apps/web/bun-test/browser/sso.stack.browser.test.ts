import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectWorkosCta, loginThroughOidc } from "../lib/flows.ts";

describe("bun.webview PoC: SSO CTAs", () => {
  beforeAll(() => {
    requireStack();
  });

  it("shows WorkOS CTA when provider mode is workos", async () => {
    expect(await expectWorkosCta()).toBe(true);
  }, 45_000);

  it("completes OIDC SSO via mock without login skeleton", async () => {
    expect(await loginThroughOidc()).toBe(true);
  }, 60_000);
});
