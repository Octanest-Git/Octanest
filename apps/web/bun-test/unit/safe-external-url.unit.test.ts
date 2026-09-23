import { describe, expect, it } from "bun:test";

import { safeExternalHttpUrl } from "../../src/lib/safe-external-url.ts";

describe("safeExternalHttpUrl (bun:test PoC)", () => {
  it("accepts http(s) URLs", () => {
    expect(safeExternalHttpUrl("https://example.com/path")).toBe("https://example.com/path");
    expect(safeExternalHttpUrl("http://example.com")).toBe("http://example.com/");
  });

  it("prefixes bare hosts with https", () => {
    expect(safeExternalHttpUrl("example.com")).toBe("https://example.com/");
  });

  it("rejects dangerous schemes and protocol-relative URLs", () => {
    expect(safeExternalHttpUrl("javascript:alert(1)")).toBeNull();
    expect(safeExternalHttpUrl("data:text/html,hi")).toBeNull();
    expect(safeExternalHttpUrl("//evil.example")).toBeNull();
  });

  it("returns null for empty or unparseable input", () => {
    expect(safeExternalHttpUrl("")).toBeNull();
    expect(safeExternalHttpUrl("   ")).toBeNull();
    expect(safeExternalHttpUrl("not a url")).toBeNull();
  });
});
