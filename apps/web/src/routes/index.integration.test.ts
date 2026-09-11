import { describe, expect, it } from "vitest";
import { Route } from "./index";

/**
 * Wave 0 (D-18/D-20): home SSR/tree gate priority.
 * Later 06-05 wires beforeLoad/loader; stubs fail until the correct tree is selected.
 */
describe("index/home SSR tree gate Wave 0 (D-18/D-20)", () => {
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
    type Input = { needs_setup: boolean; hasSession: boolean };
    type Tree = "setup" | "signed-in" | "marketing";

    function selectHomeTree(_input: Input): Tree {
      // Placeholder — wrong on purpose so Wave 0 stays RED until 06-05.
      return "marketing";
    }

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
});
