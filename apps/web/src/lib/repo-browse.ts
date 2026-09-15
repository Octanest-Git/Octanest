import type { RepoTreeEntry } from "@octanest/api-client";

/** Directories (and gitlink commits) before blobs; then localeCompare (D-15). */
export function sortTreeEntries(entries: RepoTreeEntry[]): RepoTreeEntry[] {
  return [...entries].sort((a, b) => {
    const rank = (e: RepoTreeEntry) => (e.kind === "tree" || e.kind === "commit" ? 0 : 1);
    const d = rank(a) - rank(b);
    if (d !== 0) return d;
    return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
  });
}

/** Parse `/tree/{ref}/…` or `/blob/{ref}/…` splat into ref + relative path (D-17 / WR-03).
 *
 * When `knownRefs` is provided (short branch/tag names), pick the longest matching
 * prefix of splat segments; otherwise fall back to first-segment split.
 */
export function parseRefAndPath(
  splat: string | undefined | null,
  knownRefs?: readonly string[] | null,
): {
  ref: string;
  path: string;
} {
  const parts = (splat ?? "")
    .split("/")
    .map((p) => p.trim())
    .filter(Boolean);
  if (parts.length === 0) return { ref: "", path: "" };

  const known = (knownRefs ?? []).map((r) => r.trim()).filter(Boolean);
  if (known.length > 0) {
    const knownSet = new Set(known);
    let best: { ref: string; pathSegs: number } | null = null;
    for (let i = parts.length; i >= 1; i--) {
      const candidate = parts.slice(0, i).join("/");
      if (knownSet.has(candidate)) {
        best = { ref: candidate, pathSegs: i };
        break; // longest first
      }
    }
    if (best) {
      return {
        ref: best.ref,
        path: parts.slice(best.pathSegs).join("/"),
      };
    }
  }

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

/** Parent directory path for tree `..` navigation (empty = repo root). */
export function parentRepoPath(path: string): string {
  const parts = path
    .replace(/^\/+|\/+$/g, "")
    .split("/")
    .filter(Boolean);
  if (parts.length <= 1) return "";
  return parts.slice(0, -1).join("/");
}

/** Crumb segments for tree/blob path chrome (D-17 / UI long-path backstop). */
export type PathCrumb = {
  seg: string;
  prefix: string;
  last: boolean;
};

export function pathBreadcrumbCrumbs(path: string): PathCrumb[] {
  const segments = path.split("/").filter(Boolean);
  return segments.map((seg, i) => ({
    seg,
    prefix: segments.slice(0, i + 1).join("/"),
    last: i === segments.length - 1,
  }));
}

export function treeHref(owner: string, repo: string, ref: string, path = ""): string {
  const base = `/${owner}/${repo}/tree/${encodeURIComponent(ref)}`;
  const rel = path.replace(/^\/+/, "");
  return rel ? `${base}/${rel.split("/").map(encodeURIComponent).join("/")}` : base;
}

export function blobHref(owner: string, repo: string, ref: string, path: string): string {
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

export function compareHref(owner: string, repo: string, base: string, head: string): string {
  return `/${owner}/${repo}/compare/${encodeURIComponent(base)}...${encodeURIComponent(head)}`;
}

export function blameHref(owner: string, repo: string, ref: string, path: string): string {
  const rel = path.replace(/^\/+/, "");
  return `/${owner}/${repo}/blame/${encodeURIComponent(ref)}/${rel
    .split("/")
    .map(encodeURIComponent)
    .join("/")}`;
}

export function rawBlobUrl(owner: string, repo: string, ref: string, path: string): string {
  // Same-origin relative path — SSR-safe (no window / Host needed).
  const rel = path.replace(/^\/+/, "");
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/raw/${encodeURIComponent(ref)}/${rel
    .split("/")
    .map(encodeURIComponent)
    .join("/")}`;
}

/** Human-readable byte size for blob headers (GitHub-style). */
export function formatFileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "0 Bytes";
  if (bytes < 1024) return `${Math.round(bytes)} Bytes`;
  if (bytes < 1024 * 1024) {
    const kb = bytes / 1024;
    const rounded = kb < 10 ? Math.round(kb * 10) / 10 : Math.round(kb);
    return `${rounded} KB`;
  }
  const mb = bytes / (1024 * 1024);
  const rounded = mb < 10 ? Math.round(mb * 10) / 10 : Math.round(mb);
  return `${rounded} MB`;
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
export function parseLineHash(hash: string): { start: number; end: number } | null {
  const m = hash.match(/^#?L(\d+)(?:-L?(\d+))?$/i);
  if (!m) return null;
  const start = Number(m[1]);
  const end = m[2] ? Number(m[2]) : start;
  if (!Number.isFinite(start) || start < 1) return null;
  if (!Number.isFinite(end) || end < start) return { start, end: start };
  return { start, end };
}

export function findReadmeName(entries: RepoTreeEntry[]): string | null {
  const names = entries.filter((e) => e.kind === "blob").map((e) => e.name);
  const preferred = ["README.md", "README.MD", "Readme.md", "readme.md", "README"];
  for (const p of preferred) {
    if (names.includes(p)) return p;
  }
  return names.find((n) => /^readme(\.|$)/i.test(n)) ?? null;
}

/** Short tip SHA for branch/tag lists (first 7 hex chars). */
export function shortOid(oid: string): string {
  const s = oid.trim();
  return s.length > 7 ? s.slice(0, 7) : s;
}

export function isBranchRef(fullName: string): boolean {
  return fullName.startsWith("refs/heads/");
}

export function isTagRef(fullName: string): boolean {
  return fullName.startsWith("refs/tags/");
}

export function archiveZipUrl(owner: string, repo: string, refName: string): string {
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/archive/${encodeURIComponent(refName)}.zip`;
}

export function archiveTarGzUrl(owner: string, repo: string, refName: string): string {
  return `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/archive/${encodeURIComponent(refName)}.tar.gz`;
}
