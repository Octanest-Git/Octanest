import { beforeAll, describe, expect, it } from "vitest";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    ensureAuthSettings: (patch: {
      provider_mode: "local" | "workos" | "oidc";
      email_provider: "log" | "smtp" | "resend";
      workos_client_id?: string | null;
    }) => Promise<boolean>;
    restoreLocalAuthCommand: () => Promise<boolean>;
    signupThroughUi: (creds: {
      email: string;
      username: string;
      password: string;
    }) => Promise<boolean>;
    expectWorkosCta: () => Promise<boolean>;
    loginThroughOidc: () => Promise<boolean>;
    expectStatusHealthy: () => Promise<boolean>;
    expectAuthMeDedupedOnHome: () => Promise<boolean>;
  }
}

describe("stack browser e2e: local signup + login UI", () => {
  beforeAll(() => {
    requireStack();
  });

  it("signs up through the web UI and lands on dashboard", async () => {
    await commands.ensureAuthSettings({
      provider_mode: "local",
      email_provider: "log",
    });

    const suffix = Date.now();
    const ok = await commands.signupThroughUi({
      email: `ui.user.${suffix}@octanest.local`,
      username: `uiuser${suffix}`,
      password: "password1",
    });
    expect(ok).toBe(true);
  }, 60_000);

  it("shows WorkOS CTA when provider mode is workos", async () => {
    try {
      await commands.ensureAuthSettings({
        provider_mode: "workos",
        email_provider: "log",
        workos_client_id: "client_dev_local",
      });
      const ok = await commands.expectWorkosCta();
      expect(ok).toBe(true);
    } finally {
      await commands.restoreLocalAuthCommand();
    }
  }, 45_000);

  it("completes OIDC SSO via mock without login skeleton", async () => {
    try {
      const ok = await commands.loginThroughOidc();
      expect(ok).toBe(true);
    } finally {
      await commands.restoreLocalAuthCommand();
    }
  }, 60_000);

  it("renders /status from system.health Query", async () => {
    const ok = await commands.expectStatusHealthy();
    expect(ok).toBe(true);
  }, 45_000);

  it("dedupes auth.me on signed-in home (chrome + banner share cache)", async () => {
    await commands.ensureAuthSettings({
      provider_mode: "local",
      email_provider: "log",
    });
    const ok = await commands.expectAuthMeDedupedOnHome();
    expect(ok).toBe(true);
  }, 60_000);
});
