import { describe, expect, it } from "vitest";
import { resolveAppAccessRedirect } from "./ssr-auth";

/**
 * Shared root SSR app-access gate path matrix (D-09/D-10 + UI-SPEC must_change).
 * Pure helper — no I/O; root beforeLoad consumes this.
 */
describe("resolveAppAccessRedirect path matrix (06-05)", () => {
  const appUiPaths = [
    "/login",
    "/verify",
    "/reset-password",
    "/settings/profile",
    "/admin/auth",
    "/dashboard",
    "/signup",
    "/",
  ] as const;

  it("needs_setup redirects app UI to /setup", () => {
    for (const pathname of appUiPaths) {
      expect(
        resolveAppAccessRedirect({
          pathname,
          needsSetup: true,
          mustChangeCredentials: false,
        }),
        `${pathname} must redirect to /setup while needs_setup`,
      ).toBe("/setup");
    }
  });

  it("needs_setup carve-outs: /status and /setup stay put (D-10)", () => {
    expect(
      resolveAppAccessRedirect({
        pathname: "/status",
        needsSetup: true,
        mustChangeCredentials: false,
      }),
    ).toBeNull();
    expect(
      resolveAppAccessRedirect({
        pathname: "/setup",
        needsSetup: true,
        mustChangeCredentials: false,
      }),
    ).toBeNull();
  });

  it("must_change redirects app UI to /setup/credentials", () => {
    for (const pathname of appUiPaths) {
      expect(
        resolveAppAccessRedirect({
          pathname,
          needsSetup: false,
          mustChangeCredentials: true,
        }),
        `${pathname} must redirect to /setup/credentials while must_change`,
      ).toBe("/setup/credentials");
    }
  });

  it("must_change carve-outs: /setup/credentials and /status stay put", () => {
    expect(
      resolveAppAccessRedirect({
        pathname: "/setup/credentials",
        needsSetup: false,
        mustChangeCredentials: true,
      }),
    ).toBeNull();
    expect(
      resolveAppAccessRedirect({
        pathname: "/status",
        needsSetup: false,
        mustChangeCredentials: true,
      }),
    ).toBeNull();
  });

  it("needs_setup wins over must_change when both apply", () => {
    expect(
      resolveAppAccessRedirect({
        pathname: "/login",
        needsSetup: true,
        mustChangeCredentials: true,
      }),
    ).toBe("/setup");
    expect(
      resolveAppAccessRedirect({
        pathname: "/setup/credentials",
        needsSetup: true,
        mustChangeCredentials: true,
      }),
    ).toBe("/setup");
  });

  it("no redirect when neither lock applies", () => {
    expect(
      resolveAppAccessRedirect({
        pathname: "/login",
        needsSetup: false,
        mustChangeCredentials: false,
      }),
    ).toBeNull();
  });
});
