import type { RepoTreeEntry } from "@octanest/api-client";

/** Directories (and gitlink commits) before blobs; then localeCompare (D-15). */
export function sortTreeEntries(entries: RepoTreeEntry[]): RepoTreeEntry[] {
  return [...entries].sort((a, b) => {
    const rank = (e: RepoTreeEntry) =>
      e.kind === "tree" || e.kind === "commit" ? 0 : 1;
    const d = rank(a) - rank(b);
    if (d !== 0) return d;
    return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
  });
}

/** Parse `/tree/{ref}/…` or `/blob/{ref}/…` splat into ref + relative path (D-17). */
export function parseRefAndPath(splat: string | undefined | null): {
  ref: string;
  path: string;
} {
  const parts = (splat ?? "")
    .split("/")
    .map((p) => p.trim())
    .filter(Boolean);
  if (parts.length === 0) return { ref: "", path: "" };
  return { ref: parts[0]!, path: parts.slice(1).join("/") };
}

/** Prefer short branch/tag names for URLs and Select (D-17). */
export function shortRefName(full: string): string {
  const s = full.trim();
  if (s.startsWith("refs/heads/")) return s.slice("refs/heads/".length);
  if (s.startsWith("refs/tags/")) return s.slice("refs/tags/".length);
  return s;
}

export function joinRepoPath(...parts: string[]): string {
  return parts
    .map((p) => p.replace(/^\/+|\/+$/g, ""))
    .filter(Boolean)
    .join("/");
}

export function treeHref(
  owner: string,
  repo: string,
  ref: string,
  path = "",
): string {
  const base = `/${owner}/${repo}/tree/${encodeURIComponent(ref)}`;
  const rel = path.replace(/^\/+/, "");
  return rel ? `${base}/${rel.split("/").map(encodeURIComponent).join("/")}` : base;
}

export function blobHref(
  owner: string,
  repo: string,
  ref: string,
  path: string,
): string {
  const rel = path.replace(/^\/+/, "");
  return `/${owner}/${repo}/blob/${encodeURIComponent(ref)}/${rel
    .split("/")
    .map(encodeURIComponent)
    .join("/")}`;
}

export function commitsHref(owner: string, repo: string, ref: string): string {
  return `/${owner}/${repo}/commits/${encodeURIComponent(ref)}`;
}

export function commitHref(owner: string, repo: string, sha: string): string {
  return `/${owner}/${repo}/commit/${encodeURIComponent(sha)}`;
}

export function compareHref(
  owner: string,
  repo: string,
  base: string,
  head: string,
): string {
  return `/${owner}/${repo}/compare/${encodeURIComponent(base)}...${encodeURIComponent(head)}`;
}

export function blameHref(
  owner: string,
  repo: string,
  ref: string,
  path: string,
): string {
  const rel = path.replace(/^\/+/, "");
  return `/${owner}/${repo}/blame/${encodeURIComponent(ref)}/${rel
    .split("/")
    .map(encodeURIComponent)
    .join("/")}`;
}

export function rawBlobUrl(
  owner: string,
  repo: string,
  ref: string,
  path: string,
): string {
  const origin =
    typeof window !== "undefined" ? window.location.origin : "";
  const rel = path.replace(/^\/+/, "");
  return `${origin}/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/raw/${encodeURIComponent(ref)}/${rel
    .split("/")
    .map(encodeURIComponent)
    .join("/")}`;
}

export function isImagePath(filePath: string): boolean {
  return /\.(png|jpe?g|gif|webp|svg|ico|bmp)$/i.test(filePath);
}

export function imageMimeForPath(filePath: string): string {
  const lower = filePath.toLowerCase();
  if (lower.endsWith(".png")) return "image/png";
  if (lower.endsWith(".jpg") || lower.endsWith(".jpeg")) return "image/jpeg";
  if (lower.endsWith(".gif")) return "image/gif";
  if (lower.endsWith(".webp")) return "image/webp";
  if (lower.endsWith(".svg")) return "image/svg+xml";
  if (lower.endsWith(".ico")) return "image/x-icon";
  if (lower.endsWith(".bmp")) return "image/bmp";
  return "application/octet-stream";
}

/** Parse `#L10` or `#L10-L20` line permalinks (D-20). */
export function parseLineHash(
  hash: string,
): { start: number; end: number } | null {
  const m = hash.match(/^#?L(\d+)(?:-L?(\d+))?$/i);
  if (!m) return null;
  const start = Number(m[1]);
  const end = m[2] ? Number(m[2]) : start;
  if (!Number.isFinite(start) || start < 1) return null;
  if (!Number.isFinite(end) || end < start) return { start, end: start };
  return { start, end };
}

export function findReadmeName(entries: RepoTreeEntry[]): string | null {
  const names = entries
    .filter((e) => e.kind === "blob")
    .map((e) => e.name);
  const preferred = ["README.md", "README.MD", "Readme.md", "readme.md", "README"];
  for (const p of preferred) {
    if (names.includes(p)) return p;
  }
  return names.find((n) => /^readme(\.|$)/i.test(n)) ?? null;
}
