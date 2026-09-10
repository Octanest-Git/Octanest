/**
 * D-15 / T-04-23: only same-origin relative paths starting with `/` and not `//`.
 * Homepage `/` (and empty) → `/dashboard`.
 */
export function safeReturnTo(raw: string | null | undefined): string {
  if (raw == null || raw === "") return "/dashboard";
  const value = raw.trim();
  if (!value.startsWith("/")) return "/dashboard";
  if (value.startsWith("//")) return "/dashboard";
  const lower = value.toLowerCase();
  if (
    lower.startsWith("http:") ||
    lower.startsWith("https:") ||
    lower.startsWith("javascript:") ||
    lower.startsWith("data:")
  ) {
    return "/dashboard";
  }
  if (value === "/") return "/dashboard";
  return value;
}

export function readReturnToFromLocation(): string {
  if (typeof window === "undefined") return "/dashboard";
  const params = new URLSearchParams(window.location.search);
  return safeReturnTo(params.get("returnTo"));
}
