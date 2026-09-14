/**
 * Phase 20 Wave 0 — tokens UI package:read / package:write (D-PKG-04).
 * Greened when /settings/tokens exposes classic + FG package scopes.
 */
import { describe, expect, it } from "vitest";

describe("/settings/tokens packages scopes Wave 0 stub", () => {
  it.fails("classic create offers package:read and package:write scopes", () => {
    expect(false).toBe(true);
  });

  it.fails("fine-grained create offers PackagesPerm Read/Write", () => {
    expect(false).toBe(true);
  });

  it.fails("type-to-confirm delete uses name@version before packages.deleteVersion", () => {
    // D-PKG-12 — delete confirm string contract for packages UI
    expect(false).toBe(true);
  });
});
