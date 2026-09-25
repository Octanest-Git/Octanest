import { beforeAll, describe, expect, it } from "vitest";
import { rpc, updateAuthSettings, withAdminSession } from "./client";
import { requireStack } from "./env";
import { mailpitDeleteAll, waitForMailpit } from "./mailpit";

describe("stack e2e: SMTP → Mailpit", () => {
  beforeAll(() => {
    requireStack();
  });

  it("signup welcome mail is delivered to Mailpit", async () => {
    await withAdminSession(async (adminCookie) => {
      await updateAuthSettings(adminCookie, {
        provider_mode: "local",
        email_provider: "smtp",
      });
      await mailpitDeleteAll();

      const suffix = Date.now();
      const email = `smtp.user.${suffix}@oxidean.local`;
      const username = `smtpuser${suffix}`;

      const signup = await rpc("auth.signup", {
        email,
        username,
        password: "password1",
      });
      expect(signup.ok).toBe(true);
      expect(signup.cookieHeader).toMatch(/^oxidean_session=/);

      const msg = await waitForMailpit(
        (m) =>
          m.Subject.includes("Welcome") &&
          m.To.some((t) => t.Address.toLowerCase() === email.toLowerCase()),
      );
      expect(msg.Subject).toContain("Welcome");
    });
  }, 30_000);
});
