import { beforeAll, describe, expect, it } from "vitest";
import { followRedirects, rpc, updateAuthSettings, withAdminSession } from "./client";
import { apiOrigin, requireStack } from "./env";

describe("stack e2e: OIDC → mock-oauth2-server", () => {
  beforeAll(() => {
    requireStack();
  });

  it("start → IdP → callback mints an Octanest session", async () => {
    await withAdminSession(async (adminCookie) => {
      await updateAuthSettings(adminCookie, {
        provider_mode: "oidc",
        email_provider: "log",
        oidc_issuer:
          process.env.OCTANEST_E2E_OIDC_ISSUER || "http://127.0.0.1:9090/default",
        oidc_client_id: "octanest-dev",
      });

      const start = `${apiOrigin()}/api/auth/oidc/start?return_to=${encodeURIComponent("/")}`;
      const result = await followRedirects(start, { maxHops: 16 });

      const session = result.cookies.find((c) =>
        c.startsWith("octanest_session="),
      );

      if (!session) {
        throw new Error(
          `OIDC flow did not mint session. finalUrl=${result.finalUrl} status=${result.status} body=${result.bodyText.slice(0, 400)}`,
        );
      }

      expect(new URL(result.finalUrl).pathname).toBe("/");

      const me = await rpc("auth.me", {}, session);
      expect(me.ok).toBe(true);
      const data = me.data as { email?: string; id?: string };
      expect(data.id).toBeTruthy();
      expect(data.email).toBeTruthy();
    });
  }, 60_000);
});
