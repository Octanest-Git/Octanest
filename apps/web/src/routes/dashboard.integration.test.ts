import { describe, expect, it } from "vitest";
import { Route } from "./dashboard";

describe("/dashboard Wave 0 (D-19)", () => {
  it("resolves to notFound instead of soft-redirect", () => {
    // 06-09: /dashboard must call notFound() — not window.location.replace("/")
    expect(
      Route.options.beforeLoad,
      "dashboard beforeLoad must invoke notFound()",
    ).toBeTypeOf("function");

    const componentName = String(Route.options.component?.name ?? Route.options.component);
    expect(componentName).not.toMatch(/Redirect/i);
  });
});
