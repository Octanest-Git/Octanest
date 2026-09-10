import { beforeAll, describe, expect, it } from "vitest";
import {
  adminLogin,
  followRedirects,
  rpc,
  updateAuthSettings,
} from "./client";
import { apiOrigin, requireStack } from "./env";

describe("stack e2e: OIDC → mock-oauth2-server", () => {
  beforeAll(() => {
    requireStack();
  });

  it("start → IdP → callback mints an Octanest session", async () => {
    const adminCookie = await adminLogin();
    await updateAuthSettings(adminCookie, {
      provider_mode: "oidc",
      email_provider: "log",
      oidc_issuer: process.env.OCTANEST_E2E_OIDC_ISSUER || "http://127.0.0.1:9090/default",
      oidc_client_id: "octanest-dev",
    });

    const start = `${apiOrigin()}/api/auth/oidc/start?return_to=${encodeURIComponent("/dashboard")}`;
    const result = await followRedirects(start, { maxHops: 16 });

    // Mock IdP may land on login UI or auto-approve; accept session on any hop.
    const session = result.cookies.find((c) =>
      c.startsWith("octanest_session="),
    );

    if (!session) {
      // Some mock-oauth2 builds require interactive login; surface diagnostics.
      throw new Error(
        `OIDC flow did not mint session. finalUrl=${result.finalUrl} status=${result.status} body=${result.bodyText.slice(0, 400)}`,
      );
    }

    expect(result.finalUrl).toMatch(/dashboard|localhost|127\.0\.0\.1/);

    const me = await rpc("auth.me", {}, session);
    expect(me.ok).toBe(true);
    const data = me.data as { email?: string; id?: string };
    expect(data.id).toBeTruthy();
    expect(data.email).toBeTruthy();
  }, 60_000);
});
