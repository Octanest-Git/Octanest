import { describe, expect, it } from "vitest";
import { Route, selectHomeTree } from "./index";

/**
 * Home SSR/tree gate priority (D-18/D-20).
 * Shared root owns needs_setup redirect; selectHomeTree documents D-20 priority
 * and index loader picks SignedInHome vs marketing after the root gate clears.
 */
describe("index/home SSR tree gate (D-18/D-20)", () => {
  it("registers beforeLoad/loader for needs_setup → /setup vs session → SignedInHome vs marketing", () => {
    const hasGate =
      typeof Route.options.beforeLoad === "function" ||
      typeof Route.options.loader === "function";
    expect(
      hasGate,
      "index route must gate trees via beforeLoad/loader (needs_setup | SignedInHome | marketing)",
    ).toBe(true);
  });

  it("needs_setup priority selects /setup over marketing and SignedInHome", () => {
    expect(selectHomeTree({ needs_setup: true, hasSession: false })).toBe(
      "setup",
    );
    expect(selectHomeTree({ needs_setup: false, hasSession: true })).toBe(
      "signed-in",
    );
    expect(selectHomeTree({ needs_setup: false, hasSession: false })).toBe(
      "marketing",
    );
  });

  it("SignedInHome module exists for the session tree (D-20)", async () => {
    const signedIn = await import("@/components/signed-in-home");
    expect(signedIn).toHaveProperty("SignedInHome");
    expect(
      Route.options.component,
      "index route component must exist for tree selection",
    ).toBeTruthy();
  });

  it("loader uses fetchSessionMe tree selection without client setup lock as boundary", () => {
    expect(typeof Route.options.loader).toBe("function");
    const src = Route.options.loader?.toString() ?? "";
    expect(
      /redirectIfNeedsSetup/.test(src),
      "index loader must not reintroduce client redirectIfNeedsSetup as the boundary",
    ).toBe(false);
  });
});
