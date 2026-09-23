import { describe, expect, it } from "@octanest/web/test-runner";
import { safeReturnTo } from "./return-to";

describe("safeReturnTo", () => {
  it("defaults empty to home /", () => {
    expect(safeReturnTo(null)).toBe("/");
    expect(safeReturnTo("")).toBe("/");
    expect(safeReturnTo("/")).toBe("/");
  });

  it("allows same-origin relative paths", () => {
    expect(safeReturnTo("/settings/profile")).toBe("/settings/profile");
    expect(safeReturnTo("/admin/auth?tab=1")).toBe("/admin/auth?tab=1");
  });

  it("maps legacy /dashboard to /", () => {
    expect(safeReturnTo("/dashboard")).toBe("/");
    expect(safeReturnTo("/dashboard?x=1")).toBe("/");
  });

  it("rejects open redirects and non-paths", () => {
    expect(safeReturnTo("//evil.example")).toBe("/");
    expect(safeReturnTo("https://evil.example")).toBe("/");
    expect(safeReturnTo("javascript:alert(1)")).toBe("/");
    expect(safeReturnTo("data:text/html,hi")).toBe("/");
    expect(safeReturnTo("settings")).toBe("/");
  });
});
