import { describe, expect, it } from "vitest";

/**
 * Wave 0 / D-LFS-18: blob viewer LFS badge + Download affordance.
 * Greened when BlobViewer wires parseLfsPointer (14-11).
 */
describe("BlobViewer LFS pointer (D-LFS-18)", () => {
  it.skip(
    "shows LFS badge + Download via LFS when pointer detected (AuthZ = Read)",
    async () => {
      const pointer = await import("../../lib/lfs-pointer");
      expect(pointer.parseLfsPointer).toBeTypeOf("function");

      // Production BlobViewer must expose badge + session-gated Download
      // (not PAT Basic embedded in the browser — T-14-05 / RESEARCH A2).
      const viewer = await import("./blob-viewer");
      expect(viewer.BlobViewer ?? viewer.default).toBeTruthy();
      const src = await import("./blob-viewer.tsrx?raw").then((m) =>
        String((m as { default: string }).default),
      );
      expect(src).toMatch(/LFS|lfs|parseLfsPointer/);
      expect(src).toMatch(/Download/);
    },
    30_000,
  );
});
