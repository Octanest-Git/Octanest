/**
 * Resolve a short, URL-safe build id for service-worker cache busting.
 * Prefer CI/Railway commit SHAs so deploys share a stable fingerprint.
 */
export function sanitizeSwBuildId(raw: string): string {
  const cleaned = raw
    .trim()
    .replace(/[^a-zA-Z0-9._-]/g, "")
    .slice(0, 32);
  return cleaned || "unknown";
}

export function resolveSwBuildId(
  env: NodeJS.ProcessEnv = process.env,
  fallback = (): string => `local-${Date.now().toString(36)}`,
): string {
  const candidates = [
    env.VITE_OXIDEAN_SW_BUILD,
    env.RAILWAY_GIT_COMMIT_SHA,
    env.SOURCE_COMMIT,
    env.GITHUB_SHA,
    env.COMMIT_SHA,
  ];
  for (const c of candidates) {
    if (c?.trim()) return sanitizeSwBuildId(c);
  }
  return sanitizeSwBuildId(fallback());
}

/** Placeholder embedded in apps/web/public/sw.js — replaced at build/serve time. */
export const SW_BUILD_PLACEHOLDER = "__OXIDEAN_SW_BUILD__";

export function stampSwSource(source: string, buildId: string): string {
  const id = sanitizeSwBuildId(buildId);
  if (!source.includes(SW_BUILD_PLACEHOLDER)) {
    // Already stamped or hand-edited — still force a unique CACHE_NAME suffix
    // when the file uses a static name so deploys do not pin forever.
    return source.replace(
      /const CACHE_NAME = "oxidean-shell-[^"]+";/,
      `const CACHE_NAME = "oxidean-shell-${id}";`,
    );
  }
  return source.split(SW_BUILD_PLACEHOLDER).join(id);
}
