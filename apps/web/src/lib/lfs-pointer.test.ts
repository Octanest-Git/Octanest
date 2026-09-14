import { describe, expect, it } from "vitest";
import { parseLfsPointer } from "./lfs-pointer";

/**
 * Wave 0 / D-LFS-18: pointer detection unit stubs (green in 14-11).
 */
describe("parseLfsPointer (D-LFS-18)", () => {
  it.skip("parses valid Git LFS pointer (version + oid sha256 64-hex + size)", () => {
    const text = [
      "version https://git-lfs.github.com/spec/v1",
      "oid sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "size 123",
      "",
    ].join("\n");
    const p = parseLfsPointer(text);
    expect(p).not.toBeNull();
    expect(p?.oid).toBe(
      "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    expect(p?.size).toBe(123);
  });

  it.skip("rejects binary / non-pointer text", () => {
    expect(parseLfsPointer("\0\x01binary")).toBeNull();
    expect(parseLfsPointer("hello world\n")).toBeNull();
  });

  it.skip("rejects oversized non-pointer blobs masquerading as text", () => {
    const huge = "x".repeat(1024 * 200);
    expect(parseLfsPointer(huge)).toBeNull();
  });
});
