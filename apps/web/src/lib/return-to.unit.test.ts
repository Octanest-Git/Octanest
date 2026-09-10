import { describe, expect, it } from "vitest";
import { safeReturnTo } from "./return-to";

describe("safeReturnTo", () => {
  it("defaults empty and homepage to /dashboard", () => {
    expect(safeReturnTo(null)).toBe("/dashboard");
    expect(safeReturnTo("")).toBe("/dashboard");
    expect(safeReturnTo("/")).toBe("/dashboard");
  });

  it("allows same-origin relative paths", () => {
    expect(safeReturnTo("/settings/profile")).toBe("/settings/profile");
    expect(safeReturnTo("/admin/auth?tab=1")).toBe("/admin/auth?tab=1");
  });

  it("rejects open redirects and schemes", () => {
    expect(safeReturnTo("//evil.example")).toBe("/dashboard");
    expect(safeReturnTo("https://evil.example")).toBe("/dashboard");
    expect(safeReturnTo("javascript:alert(1)")).toBe("/dashboard");
    expect(safeReturnTo("data:text/html,hi")).toBe("/dashboard");
    expect(safeReturnTo("settings")).toBe("/dashboard");
  });
});
