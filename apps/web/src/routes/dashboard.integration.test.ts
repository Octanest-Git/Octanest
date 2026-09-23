import { describe, expect, it } from "@octanest/web/test-runner";
import { isNotFound } from "@octanejs/tanstack-router";
import { Route } from "./dashboard";

describe("/dashboard Wave 0 (D-19)", () => {
  it("resolves to notFound instead of soft-redirect", async () => {
    // 06-09: /dashboard must call notFound() — not window.location.replace("/")
    expect(Route.options.beforeLoad, "dashboard beforeLoad must invoke notFound()").toBeTypeOf(
      "function",
    );

    let caught: unknown;
    try {
      await Route.options.beforeLoad!({} as never);
    } catch (e) {
      caught = e;
    }
    expect(
      isNotFound(caught),
      "beforeLoad must throw notFound() (hard 404, not soft redirect)",
    ).toBe(true);

    const componentName = String(Route.options.component?.name ?? Route.options.component);
    expect(componentName).not.toMatch(/Redirect/i);
  });
});
