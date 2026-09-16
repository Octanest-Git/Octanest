/** Copy/constants for Admin packages quota UI (D-PKG-09). */
export const DEFAULT_OWNER_QUOTA_LABEL = "10 GiB per owner";
export const DEFAULT_MAX_BLOB_LABEL = "2 GiB per blob";

/** @deprecated Prefer DEFAULT_OWNER_QUOTA_LABEL for UI; kept for tests. */
export const DEFAULT_OWNER_QUOTA_HINT =
  "OCTANEST_PACKAGES_OWNER_QUOTA_BYTES (default 10 GiB); per-owner overrides in DB";

export function packageFormatLabel(format: string): string {
  if (format === "oci") return "OCI";
  if (format === "npm") return "npm";
  if (format === "generic") return "Generic";
  return format;
}
