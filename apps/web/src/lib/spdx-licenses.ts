/** SPDX license picker options from `spdx-license-list` (D-04). */

import spdxLicenseList from "spdx-license-list";

export type SpdxLicenseOption = {
  id: string;
  label: string;
};

const NONE: SpdxLicenseOption = { id: "none", label: "None" };

let cached: SpdxLicenseOption[] | null = null;

/** Full SPDX ID list + None, sorted by id (None first). Lazy-built. */
export function listSpdxLicenseOptions(): SpdxLicenseOption[] {
  if (cached) return cached;
  const ids = Object.keys(spdxLicenseList).sort((a, b) => a.localeCompare(b));
  cached = [
    NONE,
    ...ids.map((id) => ({
      id,
      label: `${id} — ${spdxLicenseList[id]?.name ?? id}`,
    })),
  ];
  return cached;
}
