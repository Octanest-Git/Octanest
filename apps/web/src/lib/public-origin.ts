/**
 * Browser-facing site origin for clone URLs and other absolute links.
 * Prefer configured public origin (same knob as API magic links).
 */
export function resolvePublicOriginFromEnv(): string | null {
  const configured =
    process.env.OCTANEST_PUBLIC_ORIGIN?.trim() ||
    process.env.OCTANEST_COMPOSE_PUBLIC_ORIGIN?.trim() ||
    "";
  if (configured) {
    return configured.replace(/\/$/, "");
  }
  return null;
}

/** Client-safe fallback when store/loader origin is missing (tests / edge). */
export function resolvePublicOriginClient(): string {
  if (typeof window !== "undefined" && window.location?.origin) {
    return window.location.origin.replace(/\/$/, "");
  }
  return resolvePublicOriginFromEnv() ?? "http://localhost";
}

/** HTTPS clone remote for a repo (absolute — git clients need a full URL). */
export function httpsCloneUrl(
  origin: string,
  owner: string,
  repo: string,
): string {
  const base = (origin || resolvePublicOriginClient()).replace(/\/$/, "");
  return `${base}/${owner}/${repo}.git`;
}
