/**
 * Git LFS pointer detection (D-LFS-18).
 * Spec: https://github.com/git-lfs/git-lfs/blob/main/docs/spec.md
 */

export type LfsPointer = {
  version: string;
  oid: string;
  size: number;
};

const MAX_POINTER_BYTES = 1024;
const OID_RE = /^[0-9a-f]{64}$/;

/**
 * Parse Git LFS pointer text. Returns null when content is not a valid pointer
 * (binary, oversized, missing version/oid/size lines).
 */
export function parseLfsPointer(text: string): LfsPointer | null {
  if (typeof text !== "string") return null;
  // Soft size gate — pointers are tiny; reject large blobs early.
  if (text.length > MAX_POINTER_BYTES) return null;
  // Reject NULs / obvious binary.
  if (text.includes("\0")) return null;

  const lines = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
  // Allow a single trailing empty line after required fields.
  while (lines.length > 0 && lines[lines.length - 1] === "") {
    lines.pop();
  }
  if (lines.length < 3) return null;

  const versionLine = lines[0]!;
  if (!versionLine.startsWith("version ")) return null;
  const version = versionLine.slice("version ".length).trim();
  if (!version.startsWith("https://git-lfs.github.com/spec/v1")) return null;

  let oid: string | null = null;
  let size: number | null = null;

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i]!;
    if (line.startsWith("oid sha256:")) {
      const hex = line.slice("oid sha256:".length).trim();
      if (!OID_RE.test(hex)) return null;
      oid = hex;
      continue;
    }
    if (line.startsWith("size ")) {
      const rest = line.slice("size ".length).trim();
      if (!/^\d+$/.test(rest)) return null;
      size = Number(rest);
      if (!Number.isSafeInteger(size) || size < 0) return null;
      continue;
    }
    // Unknown keys allowed after required fields in some clients — reject
    // anything that breaks the three-line minimum contract for Phase 14.
    return null;
  }

  if (oid === null || size === null) return null;
  return { version, oid, size };
}
