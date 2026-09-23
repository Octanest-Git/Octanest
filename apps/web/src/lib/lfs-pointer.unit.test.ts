import { describe, expect, it } from "@octanest/web/test-runner";
import { parseLfsPointer } from "./lfs-pointer";

/**
 * D-LFS-18: pointer detection unit tests.
 */
describe("parseLfsPointer (D-LFS-18)", () => {
  it("parses valid Git LFS pointer (version + oid sha256 64-hex + size)", () => {
    const text = [
      "version https://git-lfs.github.com/spec/v1",
      "oid sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "size 123",
      "",
    ].join("\n");
    const p = parseLfsPointer(text);
    expect(p).not.toBeNull();
    expect(p?.oid).toBe("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    expect(p?.size).toBe(123);
  });

  it("rejects binary / non-pointer text", () => {
    expect(parseLfsPointer("\0\x01binary")).toBeNull();
    expect(parseLfsPointer("hello world\n")).toBeNull();
  });

  it("rejects oversized non-pointer blobs masquerading as text", () => {
    const huge = "x".repeat(1024 * 200);
    expect(parseLfsPointer(huge)).toBeNull();
  });
});
