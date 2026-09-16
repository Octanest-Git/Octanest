/** WCAG 2.1 relative luminance + contrast helpers for token / a11y tests. */

function parseHex(hex: string): [number, number, number] {
  const h = hex.replace("#", "").trim();
  const full =
    h.length === 3
      ? h
          .split("")
          .map((c) => c + c)
          .join("")
      : h;
  if (full.length !== 6 || !/^[0-9a-fA-F]+$/.test(full)) {
    throw new Error(`invalid hex color: ${hex}`);
  }
  return [
    Number.parseInt(full.slice(0, 2), 16),
    Number.parseInt(full.slice(2, 4), 16),
    Number.parseInt(full.slice(4, 6), 16),
  ];
}

function srgbChannelToLinear(c8: number): number {
  const c = c8 / 255;
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

/** Relative luminance (0–1) for an sRGB hex color. */
export function relativeLuminance(hex: string): number {
  const [r, g, b] = parseHex(hex);
  const R = srgbChannelToLinear(r);
  const G = srgbChannelToLinear(g);
  const B = srgbChannelToLinear(b);
  return 0.2126 * R + 0.7152 * G + 0.0722 * B;
}

/** Contrast ratio between two hex colors (WCAG). Always ≥ 1. */
export function contrastRatio(foregroundHex: string, backgroundHex: string): number {
  const L1 = relativeLuminance(foregroundHex);
  const L2 = relativeLuminance(backgroundHex);
  const lighter = Math.max(L1, L2);
  const darker = Math.min(L1, L2);
  return (lighter + 0.05) / (darker + 0.05);
}

export function meetsWcagAa(
  foregroundHex: string,
  backgroundHex: string,
  largeText = false,
): boolean {
  const min = largeText ? 3 : 4.5;
  return contrastRatio(foregroundHex, backgroundHex) >= min;
}

export function meetsWcagAaUi(foregroundHex: string, backgroundHex: string): boolean {
  return contrastRatio(foregroundHex, backgroundHex) >= 3;
}

/**
 * Canonical palette pairs under test (must stay in sync with `styles.css` tokens).
 * Keep literal hex here so tests fail if CSS drifts without updating expectations.
 */
export const THEME_CONTRAST_PAIRS = {
  light: {
    foregroundOnBackground: ["#1f2328", "#ffffff"] as const,
    mutedOnBackground: ["#59636e", "#ffffff"] as const,
    primaryOnBackground: ["#0969da", "#ffffff"] as const,
    primaryFgOnPrimary: ["#ffffff", "#0969da"] as const,
    destructiveFgOnDestructive: ["#ffffff", "#cf222e"] as const,
  },
  dark: {
    foregroundOnBackground: ["#f0f3f6", "#010409"] as const,
    mutedOnBackground: ["#9da7b3", "#010409"] as const,
    primaryOnBackground: ["#58a6ff", "#010409"] as const,
    primaryFgOnPrimary: ["#010409", "#58a6ff"] as const,
    cardFgOnCard: ["#f0f3f6", "#0d1117"] as const,
  },
} as const;
