import { beforeAll, describe, expect, it } from "vitest";
import { page } from "vitest/browser";
import { adminLogin, updateAuthSettings } from "../stack/client";
import { requireStack, webOrigin } from "../stack/env";

describe("stack browser e2e: local signup + login UI", () => {
  beforeAll(async () => {
    requireStack();
    const cookie = await adminLogin();
    await updateAuthSettings(cookie, {
      provider_mode: "local",
      email_provider: "log",
    });
  });

  it("signs up through the web UI and lands on dashboard", async () => {
    const suffix = Date.now();
    const email = `ui.user.${suffix}@octanest.local`;
    const username = `uiuser${suffix}`;

    await page.goto(`${webOrigin()}/signup`);
    await expect
      .element(page.getByRole("heading", { name: "Create your account" }))
      .toBeVisible();

    await page.getByLabelText("Email").fill(email);
    await page.getByLabelText("Username").fill(username);
    await page.getByLabelText("Password", { exact: true }).fill("password1");
    await page.getByLabelText(/confirm password/i).fill("password1");

    await page.getByRole("button", { name: /create account|sign up/i }).click();

    await expect.poll(() => new URL(page.url()).pathname).toBe("/dashboard");
  }, 60_000);

  it("shows WorkOS CTA when provider mode is workos", async () => {
    const cookie = await adminLogin();
    await updateAuthSettings(cookie, {
      provider_mode: "workos",
      email_provider: "log",
      workos_client_id: "client_dev_local",
    });

    await page.goto(`${webOrigin()}/login`);
    await expect
      .element(page.getByRole("button", { name: /continue with workos/i }))
      .toBeVisible();
  }, 45_000);
});
