import { beforeAll, describe, expect, it } from "vitest";
import { followRedirects, rpc, updateAuthSettings, withAdminSession } from "./client";
import { apiOrigin, requireStack } from "./env";
import { stubsReset, waitForStub } from "./stubs";

describe("stack e2e: WorkOS → AuthKit stub", () => {
  beforeAll(() => {
    requireStack();
  });

  it("start → stub authorize → callback mints session for stub user", async () => {
    await withAdminSession(async (adminCookie) => {
      await updateAuthSettings(adminCookie, {
        provider_mode: "workos",
        email_provider: "log",
        workos_client_id: "client_dev_local",
      });
      await stubsReset();

      const start = `${apiOrigin()}/api/auth/workos/start?return_to=${encodeURIComponent("/")}`;
      const result = await followRedirects(start);

      expect(new URL(result.finalUrl).pathname).toBe("/");
      const session = result.cookies.find((c) => c.startsWith("oxidean_session="));
      expect(session).toBeTruthy();

      await waitForStub((e) => e.method === "POST" && e.path === "/user_management/authenticate");

      const me = await rpc("auth.me", {}, session);
      expect(me.ok).toBe(true);
      const data = me.data as { email?: string };
      expect(data.email).toBe("dev@oxidean.local");
    });
  }, 45_000);
});
