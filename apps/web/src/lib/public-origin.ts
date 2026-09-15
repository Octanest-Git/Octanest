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
export function httpsCloneUrl(origin: string, owner: string, repo: string): string {
  const base = (origin || resolvePublicOriginClient()).replace(/\/$/, "");
  return `${base}/${owner}/${repo}.git`;
}

/**
 * scp-style SSH clone URL (D-SSH-02).
 * Always `git@{host}:{owner}/{repo}.git` — port is never embedded; use
 * `sshNeedsPortHint` / `~/.ssh/config Port` when advertised port ≠ 22.
 */
export function sshCloneUrl(host: string, _port: number, owner: string, repo: string): string {
  const h = host
    .trim()
    .replace(/\/$/, "")
    .replace(/^\[|\]$/g, "");
  return `git@${h}:${owner}/${repo}.git`;
}

/** True when clients need an explicit SSH Port (not the default 22). */
export function sshNeedsPortHint(port: number): boolean {
  return Number.isFinite(port) && port > 0 && port !== 22;
}

/** Advertised SSH hostname (env or hostname of public origin). */
export function resolveSshHost(
  publicOrigin?: string,
  envHost = process.env.OCTANEST_SSH_HOST,
): string {
  const fromEnv = envHost?.trim();
  if (fromEnv) return fromEnv.replace(/\/$/, "");
  const origin =
    (publicOrigin || "").trim() || resolvePublicOriginFromEnv() || resolvePublicOriginClient();
  try {
    const u = new URL(origin.includes("://") ? origin : `http://${origin}`);
    return u.hostname || "localhost";
  } catch {
    return "localhost";
  }
}

/** Advertised/listen SSH port (env default 2222 for Compose). */
export function resolveSshPort(envPort = process.env.OCTANEST_SSH_PORT): number {
  const raw = envPort?.trim();
  if (raw) {
    const n = Number.parseInt(raw, 10);
    if (Number.isFinite(n) && n > 0) return n;
  }
  return 2222;
}
