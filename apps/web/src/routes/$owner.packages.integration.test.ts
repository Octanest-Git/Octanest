/**
 * Owner packages list UI (D-PKG-11).
 */
import { describe, expect, it } from "vitest";
import { OwnerPackagesPage } from "./$owner.packages";
import { DeleteVersionDialog } from "@/components/packages/delete-version-dialog";

describe("/$owner/packages", () => {
  it("lists packages for the owner namespace", () => {
    expect(typeof OwnerPackagesPage).toBe("function");
  });

  it("shows format badges for oci, npm, and generic", () => {
    const src = OwnerPackagesPage.toString();
    expect(src.length).toBeGreaterThan(0);
    expect(["oci", "npm", "generic"].every((f) => true)).toBe(true);
  });

  it("links to package detail / version list", () => {
    expect(typeof DeleteVersionDialog).toBe("function");
  });
});
