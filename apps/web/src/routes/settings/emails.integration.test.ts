/**
 * /settings/emails redirects to Account; coverage stays on profile.integration.test.ts.
 */
import { describe, expect, it } from "vitest";

describe("/settings/emails redirect", () => {
  it("exports a Route that redirects to /settings/profile", async () => {
    const mod = await import("./emails");
    expect(mod.Route).toBeTruthy();
    expect(mod.Route.options.beforeLoad).toBeTypeOf("function");
  });
});
