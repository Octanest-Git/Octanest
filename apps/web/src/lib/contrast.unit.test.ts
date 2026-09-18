import { describe, expect, it } from "vitest";
import {
  THEME_CONTRAST_PAIRS,
  contrastRatio,
  meetsWcagAa,
  meetsWcagAaUi,
  relativeLuminance,
} from "./contrast";

describe("contrast helpers", () => {
  it("computes black/white as 21:1", () => {
    expect(contrastRatio("#000000", "#ffffff")).toBeCloseTo(21, 0);
    expect(relativeLuminance("#ffffff")).toBeCloseTo(1, 3);
    expect(relativeLuminance("#000000")).toBeCloseTo(0, 3);
  });

  it("meets AA for light theme body and muted text", () => {
    const { light } = THEME_CONTRAST_PAIRS;
    expect(meetsWcagAa(...light.foregroundOnBackground)).toBe(true);
    expect(meetsWcagAa(...light.mutedOnBackground)).toBe(true);
    expect(meetsWcagAaUi(...light.primaryOnBackground)).toBe(true);
    expect(meetsWcagAa(...light.primaryFgOnPrimary)).toBe(true);
    expect(meetsWcagAa(...light.destructiveFgOnDestructive)).toBe(true);
  });

  it("meets AA for deeper dark theme body and muted text", () => {
    const { dark } = THEME_CONTRAST_PAIRS;
    expect(meetsWcagAa(...dark.foregroundOnBackground)).toBe(true);
    expect(meetsWcagAa(...dark.mutedOnBackground)).toBe(true);
    expect(meetsWcagAaUi(...dark.primaryOnBackground)).toBe(true);
    expect(meetsWcagAa(...dark.primaryFgOnPrimary)).toBe(true);
    expect(meetsWcagAa(...dark.cardFgOnCard)).toBe(true);
    // True OLED-black canvas is deeper than classic #0d1117
    expect(relativeLuminance("#000000")).toBeLessThan(relativeLuminance("#0d1117"));
  });

  it("reports known weak pairs as failing AA", () => {
    expect(meetsWcagAa("#777777", "#666666")).toBe(false);
    expect(meetsWcagAa("#888888", "#777777")).toBe(false);
  });

  it("keeps styles.css dark canvas OLED-black and muted text AA", async () => {
    const fs = await import("node:fs");
    const path = await import("node:path");
    const url = await import("node:url");
    const cssPath = path.join(path.dirname(url.fileURLToPath(import.meta.url)), "../styles.css");
    const css = fs.readFileSync(cssPath, "utf8");
    expect(css).toMatch(/\.dark\s*\{[^}]*--background:\s*#000000/s);
    expect(css).toMatch(/\.dark\s*\{[^}]*--muted-foreground:\s*#9da7b3/s);
    expect(css).not.toMatch(/feTurbulence/);
    expect(css).toMatch(/:focus-visible/);
  });
});
