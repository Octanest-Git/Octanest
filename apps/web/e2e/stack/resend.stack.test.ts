import { beforeAll, describe, expect, it } from "vitest";
import { rpc, updateAuthSettings, withAdminSession } from "./client";
import { requireStack } from "./env";
import { stubsReset, waitForStub } from "./stubs";

describe("stack e2e: Resend → HTTP stub", () => {
  beforeAll(() => {
    requireStack();
  });

  it("signup welcome mail POSTs to Resend stub", async () => {
    await withAdminSession(async (adminCookie) => {
      await updateAuthSettings(adminCookie, {
        provider_mode: "local",
        email_provider: "resend",
      });
      await stubsReset();

      const suffix = Date.now();
      const email = `resend.user.${suffix}@octanest.local`;
      const username = `resenduser${suffix}`;

      const signup = await rpc("auth.signup", {
        email,
        username,
        password: "password1",
      });
      expect(signup.ok).toBe(true);

      const entry = await waitForStub(
        (e) => e.method === "POST" && e.path === "/emails",
      );
      const body = entry.body as {
        to?: string[];
        subject?: string;
        from?: string;
      };
      expect(body.to?.[0]?.toLowerCase()).toBe(email.toLowerCase());
      expect(body.subject).toMatch(/Welcome/i);
    });
  }, 30_000);
});
