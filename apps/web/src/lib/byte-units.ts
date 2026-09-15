/** Binary byte units for LFS / storage quota inputs (powers of 1024). */
export type ByteUnit = "B" | "KiB" | "MiB" | "GiB" | "TiB";

export const BYTE_UNIT_FACTORS: Record<ByteUnit, number> = {
  B: 1,
  KiB: 1024,
  MiB: 1024 ** 2,
  GiB: 1024 ** 3,
  TiB: 1024 ** 4,
};

export const BYTE_UNIT_OPTIONS: { value: ByteUnit; label: string }[] = [
  { value: "B", label: "B" },
  { value: "KiB", label: "KiB" },
  { value: "MiB", label: "MiB" },
  { value: "GiB", label: "GiB" },
  { value: "TiB", label: "TiB" },
];

const UNIT_ORDER: ByteUnit[] = ["TiB", "GiB", "MiB", "KiB", "B"];

/** Human-readable size (binary prefixes). */
export function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "0 B";
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GiB`;
}

/**
 * Prefer the largest unit that yields an integer or single-decimal display.
 * Zero defaults to GiB (instance quotas are typically expressed in GiB).
 */
export function pickDisplayUnit(bytes: number): ByteUnit {
  if (!Number.isFinite(bytes) || bytes === 0) return "GiB";
  for (const unit of UNIT_ORDER) {
    if (unit === "B") return "B";
    const value = bytes / BYTE_UNIT_FACTORS[unit];
    if (value < 1) continue;
    if (Number.isInteger(value)) return unit;
    if (Math.abs(value * 10 - Math.round(value * 10)) < 1e-9) return unit;
  }
  for (const unit of UNIT_ORDER) {
    if (bytes / BYTE_UNIT_FACTORS[unit] >= 1) return unit;
  }
  return "B";
}

/** Format bytes as a compact numeric string in `unit` (no unit suffix). */
export function bytesToUnitString(bytes: number, unit: ByteUnit): string {
  if (!Number.isFinite(bytes)) return "";
  const value = bytes / BYTE_UNIT_FACTORS[unit];
  if (Number.isInteger(value)) return String(value);
  const rounded = Math.round(value * 1000) / 1000;
  return String(rounded);
}

/** Parse a display amount + unit into whole bytes, or null if invalid. */
export function unitValueToBytes(raw: string, unit: ByteUnit): number | null {
  const trimmed = raw.trim();
  if (trimmed === "") return null;
  const n = Number(trimmed);
  if (!Number.isFinite(n) || n < 0) return null;
  const bytes = n * BYTE_UNIT_FACTORS[unit];
  if (!Number.isFinite(bytes)) return null;
  return Math.round(bytes);
}

/** Keep the same byte amount when the operator switches units. */
export function convertUnitDisplay(raw: string, from: ByteUnit, to: ByteUnit): string {
  if (from === to) return raw;
  const bytes = unitValueToBytes(raw, from);
  if (bytes === null) return raw;
  return bytesToUnitString(bytes, to);
}

export type LfsUsageChartRow = {
  label: string;
  logicalBytes: number;
  /** Logical size in GiB for chart axis (3 decimal places). */
  giB: number;
};

/** Top-N usage rows sorted by logical bytes for bar charts. */
export function toLfsUsageChartRows(
  rows: ReadonlyArray<{ label: string; logicalBytes: number }>,
  limit = 8,
): LfsUsageChartRow[] {
  return rows
    .slice()
    .sort((a, b) => b.logicalBytes - a.logicalBytes)
    .slice(0, Math.max(0, limit))
    .map((row) => ({
      label: row.label,
      logicalBytes: row.logicalBytes,
      giB: Number((row.logicalBytes / BYTE_UNIT_FACTORS.GiB).toFixed(3)),
    }));
}
