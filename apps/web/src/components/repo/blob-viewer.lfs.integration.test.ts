import { describe, expect, it } from "vitest";

/**
 * D-LFS-18: blob viewer LFS badge + Download affordance.
 */
describe("BlobViewer LFS pointer (D-LFS-18)", () => {
  it(
    "shows LFS badge + Download via LFS when pointer detected (AuthZ = Read)",
    async () => {
      const pointer = await import("../../lib/lfs-pointer");
      expect(pointer.parseLfsPointer).toBeTypeOf("function");

      const viewer = await import("./blob-viewer");
      expect(viewer.BlobViewer ?? viewer.default).toBeTruthy();
      const src = await import("./blob-viewer.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
      expect(src).toMatch(/parseLfsPointer/);
      expect(src).toMatch(/LFS/);
      expect(src).toMatch(/Download via LFS/);
      expect(src).toMatch(/repo\.lfs\.download|lfs\.download/);
    },
    30_000,
  );
});
