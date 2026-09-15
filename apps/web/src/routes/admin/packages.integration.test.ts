/**
 * Admin packages quota/usage (D-PKG-09) + G-11.1-15 render mount.
 * Typeof / DEFAULT_OWNER_QUOTA_HINT-only checks alone are insufficient.
 */
import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";
import { DEFAULT_OWNER_QUOTA_HINT } from "@/lib/package-quota-copy";

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(),
    },
    packages: {
      adminUsage: vi.fn(),
      adminSetQuota: vi.fn(),
    },
  },
}));

import { AdminPackagesPage } from "./packages";

afterEach(cleanup);

describe("/admin/packages", () => {
  it("exports AdminPackagesPage and keeps owner-quota copy hints", () => {
    expect(typeof AdminPackagesPage).toBe("function");
    expect(DEFAULT_OWNER_QUOTA_HINT.toLowerCase()).toContain("quota");
    expect(DEFAULT_OWNER_QUOTA_HINT).toContain(
      "OCTANEST_PACKAGES_OWNER_QUOTA_BYTES",
    );
  });

  it("renders Package storage for admin without throwing (G-11.1-15)", async () => {
    renderWithQueryClient(AdminPackagesPage);

    await waitFor(
      () => {
        expect(screen.getByTestId("admin-packages")).toBeTruthy();
        expect(screen.getByText("Package storage")).toBeTruthy();
        expect(screen.getByText("Max blob size")).toBeTruthy();
        expect(screen.getByText("Per-owner quota")).toBeTruthy();
      },
      { timeout: 10_000 },
    );
  });
});
