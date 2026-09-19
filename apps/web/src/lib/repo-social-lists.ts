/** Helpers for repo social list pages (stargazers / watchers / forks). */

export type ForksSortOption = "stars" | "updated" | "created";

export function parseForksSort(raw: string | null | undefined): ForksSortOption {
  const v = String(raw ?? "")
    .trim()
    .toLowerCase();
  if (v === "updated" || v === "recently_updated") return "updated";
  if (v === "created" || v === "recently_created") return "created";
  return "stars";
}

export function forksSortLabel(sort: ForksSortOption): string {
  switch (sort) {
    case "updated":
      return "Recently updated";
    case "created":
      return "Recently created";
    default:
      return "Most starred";
  }
}

/** Build list URL preserving owner/repo and optional q/sort query params. */
export function socialListHref(
  base: string,
  opts: { q?: string; sort?: string; page?: number } = {},
): string {
  const params = new URLSearchParams();
  const q = (opts.q ?? "").trim();
  if (q) params.set("q", q);
  if (opts.sort && opts.sort !== "stars") params.set("sort", opts.sort);
  if (opts.page && opts.page > 1) params.set("page", String(opts.page));
  const qs = params.toString();
  return qs ? `${base}?${qs}` : base;
}
