/**
 * D-15 / T-04-23: only same-origin relative paths starting with `/` and not `//`.
 * Empty / invalid → `/` (signed-in home). Legacy `/dashboard` → `/`.
 */
export function safeReturnTo(raw: string | null | undefined): string {
  if (raw == null || raw === "") return "/";
  const value = raw.trim();
  if (!value.startsWith("/")) return "/";
  if (value.startsWith("//")) return "/";
  const lower = value.toLowerCase();
  if (
    lower.startsWith("http:") ||
    lower.startsWith("https:") ||
    lower.startsWith("javascript:") ||
    lower.startsWith("data:")
  ) {
    return "/";
  }
  if (value === "/dashboard" || value.startsWith("/dashboard?")) {
    return "/";
  }
  return value;
}

export function readReturnToFromLocation(): string {
  if (typeof window === "undefined") return "/";
  const params = new URLSearchParams(window.location.search);
  return safeReturnTo(params.get("returnTo"));
}
