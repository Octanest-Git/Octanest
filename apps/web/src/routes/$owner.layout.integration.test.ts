/**
 * Regression: `/$owner` is a layout for repos/packages; org overview is index-only.
 * Parent must not notFound() for user accounts or `/{user}/{repo}` never renders.
 */
import { describe, expect, it } from "vitest";

describe("/$owner layout vs org index", () => {
  it(
    "layout has no org notFound loader; index owns org overview",
    async () => {
      const layout = await import("./$owner.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
      const index = await import("./$owner.index.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
      expect(layout).toMatch(/Outlet/);
      expect(layout).not.toMatch(/notFound\(/);
      expect(layout).not.toMatch(/fetchOrgOverview/);
      expect(index).toMatch(/fetchOrgOverview/);
      expect(index).toMatch(/notFound\(/);
      expect(index).toMatch(/OrgOverviewPage/);
    },
    30_000,
  );
});
