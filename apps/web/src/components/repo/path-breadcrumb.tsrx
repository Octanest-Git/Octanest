import { treeHref } from "@/lib/repo-browse";

type PathBreadcrumbProps = {
  owner: string;
  repo: string;
  refName: string;
  path: string;
};

/** Label-size path crumbs — each segment links to tree (D-17). */
export function PathBreadcrumb({
  owner,
  repo,
  refName,
  path,
}: PathBreadcrumbProps) @{
  const segments = path.split("/").filter(Boolean);
  const crumbs = segments.map((seg, i) => ({
    seg,
    i,
    prefix: segments.slice(0, i + 1).join("/"),
    last: i === segments.length - 1,
  }));
  const rootHref = treeHref(owner, repo, refName, "");

  <nav aria-label="Path" className="mb-4 flex flex-wrap items-center gap-1 text-[14px]">
    <a
      href={rootHref as string}
      className="font-semibold text-primary no-underline hover:underline"
    >
      {repo as string}
    </a>
    @for (const c of crumbs; key `${c.i}-${c.seg}`) {
      const href = treeHref(owner, repo, refName, c.prefix);
      <span className="text-muted-foreground" aria-hidden>
        /
      </span>
      @if (c.last) {
        <span className="font-semibold text-foreground">{c.seg as string}</span>
      } @else {
        <a
          href={href as string}
          className="font-normal text-primary no-underline hover:underline"
        >
          {c.seg as string}
        </a>
      }
    }
  </nav>
}
