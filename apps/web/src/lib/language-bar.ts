/**
 * GitHub-style language % labels: one decimal place; tiny shares show as `<0.1%`.
 * Bar segment widths should still use raw byte ratios, not these rounded labels.
 */
export function languagePercent(bytes: number, total: number): string {
  if (total <= 0 || bytes <= 0) return "0%";
  const pct = (bytes / total) * 100;
  if (pct < 0.1) return "<0.1%";
  return `${pct.toFixed(1)}%`;
}
