/**
 * Git LFS pointer detection (D-LFS-18).
 * Wave 0 stub — real parse greened in 14-11.
 */

export type LfsPointer = {
  version: string;
  oid: string;
  size: number;
};

/**
 * Parse Git LFS pointer text. Returns null when content is not a valid pointer
 * (binary, oversized, missing version/oid/size lines).
 */
export function parseLfsPointer(_text: string): LfsPointer | null {
  // Wave 0: stub returns null until 14-11 implements pointer rules.
  return null;
}
