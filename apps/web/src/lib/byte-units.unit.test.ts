import { describe, expect, it } from "vitest";
import {
  BYTE_UNIT_FACTORS,
  bytesToUnitString,
  convertUnitDisplay,
  formatBytes,
  pickDisplayUnit,
  toLfsUsageChartRows,
  unitValueToBytes,
} from "./byte-units";

describe("byte-units (LFS quota inputs)", () => {
  it("formatBytes uses binary prefixes", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(2048)).toBe("2.0 KiB");
    expect(formatBytes(BYTE_UNIT_FACTORS.GiB)).toBe("1.00 GiB");
  });

  it("pickDisplayUnit prefers exact larger units", () => {
    expect(pickDisplayUnit(0)).toBe("GiB");
    expect(pickDisplayUnit(1024)).toBe("KiB");
    expect(pickDisplayUnit(10 * BYTE_UNIT_FACTORS.GiB)).toBe("GiB");
    expect(pickDisplayUnit(1.5 * BYTE_UNIT_FACTORS.MiB)).toBe("MiB");
    expect(pickDisplayUnit(3)).toBe("B");
  });

  it("round-trips display strings through unitValueToBytes", () => {
    expect(unitValueToBytes("10", "GiB")).toBe(10 * BYTE_UNIT_FACTORS.GiB);
    expect(unitValueToBytes("2.5", "MiB")).toBe(Math.round(2.5 * BYTE_UNIT_FACTORS.MiB));
    expect(unitValueToBytes("", "GiB")).toBeNull();
    expect(unitValueToBytes("-1", "GiB")).toBeNull();
    expect(unitValueToBytes("nope", "B")).toBeNull();
  });

  it("bytesToUnitString trims trailing noise", () => {
    expect(bytesToUnitString(10 * BYTE_UNIT_FACTORS.GiB, "GiB")).toBe("10");
    expect(bytesToUnitString(1536, "KiB")).toBe("1.5");
  });

  it("convertUnitDisplay preserves byte amount across unit changes", () => {
    expect(convertUnitDisplay("1024", "KiB", "MiB")).toBe("1");
    expect(convertUnitDisplay("1", "GiB", "MiB")).toBe("1024");
    expect(convertUnitDisplay("abc", "GiB", "MiB")).toBe("abc");
  });

  it("toLfsUsageChartRows sorts, limits, and exposes GiB", () => {
    const rows = toLfsUsageChartRows(
      [
        { label: "a/small", logicalBytes: 100 },
        { label: "b/big", logicalBytes: 3 * BYTE_UNIT_FACTORS.GiB },
        { label: "c/mid", logicalBytes: BYTE_UNIT_FACTORS.GiB },
      ],
      2,
    );
    expect(rows).toHaveLength(2);
    expect(rows[0]?.label).toBe("b/big");
    expect(rows[0]?.giB).toBe(3);
    expect(rows[1]?.label).toBe("c/mid");
    expect(rows[1]?.giB).toBe(1);
  });
});
