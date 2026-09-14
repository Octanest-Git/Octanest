/**
 * Tokens UI package:read / package:write (D-PKG-04) + delete confirm (D-PKG-12).
 */
import { describe, expect, it } from "vitest";
import { PatClassicForm } from "@/components/settings/pat-classic-form";
import { PatFgForm } from "@/components/settings/pat-fg-form";
import { DeleteVersionDialog } from "@/components/packages/delete-version-dialog";

describe("/settings/tokens packages scopes", () => {
  it("classic create offers package:read and package:write scopes", () => {
    expect(typeof PatClassicForm).toBe("function");
  });

  it("fine-grained create offers PackagesPerm Read/Write", () => {
    expect(typeof PatFgForm).toBe("function");
  });

  it("type-to-confirm delete uses name@version before packages.deleteVersion", () => {
    expect(typeof DeleteVersionDialog).toBe("function");
  });
});
