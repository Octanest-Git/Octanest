/**
 * Allow only http(s) absolute URLs for user-supplied links (e.g. repo homepage).
 * Rejects javascript:/data:/protocol-relative and unparseable values.
 * Bare hosts (example.com) are treated as https://example.com.
 */
export function safeExternalHttpUrl(raw: string | null | undefined): string | null {
  const value = (raw ?? "").trim();
  if (!value) return null;
  if (value.startsWith("//")) return null;
  const candidate = value.includes("://") ? value : `https://${value}`;
  try {
    const parsed = new URL(candidate);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return null;
    if (!parsed.hostname) return null;
    return parsed.href;
  } catch {
    return null;
  }
}
