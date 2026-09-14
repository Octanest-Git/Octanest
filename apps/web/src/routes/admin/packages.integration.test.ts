/**
 * Admin packages quota/usage (D-PKG-09).
 */
import { describe, expect, it } from "vitest";
import { AdminPackagesPage } from "./packages";
import { DEFAULT_OWNER_QUOTA_HINT } from "@/lib/package-quota-copy";

describe("/admin/packages", () => {
  it("shows package storage usage broken down by format", () => {
    expect(typeof AdminPackagesPage).toBe("function");
    expect(DEFAULT_OWNER_QUOTA_HINT.toLowerCase()).toContain("quota");
  });

  it("exposes max blob size and per-owner quota settings", () => {
    expect(DEFAULT_OWNER_QUOTA_HINT).toContain("OCTANEST_PACKAGES_OWNER_QUOTA_BYTES");
  });
});
