import type { RepoTreeEntry } from "@octanest/api-client";
import { FileCode2, Folder, Package } from "@octanejs/lucide";
import { blobHref, joinRepoPath, sortTreeEntries, treeHref } from "@/lib/repo-browse";

type FileTreeProps = {
  owner: string;
  repo: string;
  refName: string;
  basePath: string;
  entries: RepoTreeEntry[];
};

/** Sorted directory listing — dirs/gitlinks first (D-15, D-21). */
export function FileTree({ owner, repo, refName, basePath, entries }: FileTreeProps) @{
  const sorted = sortTreeEntries(entries);

  <ul
    role="list"
    aria-label="Directories and files"
    className="divide-y divide-border overflow-y-auto rounded-md border border-border bg-card/40"
  >
    @for (const entry of sorted; key entry.oid + entry.name) {
      const childPath = joinRepoPath(basePath, entry.name);
      const isTree = entry.kind === "tree";
      const isLink = entry.kind === "commit";
      const href = isTree
        ? treeHref(owner, repo, refName, childPath)
        : isLink
          ? treeHref(owner, repo, refName, childPath)
          : blobHref(owner, repo, refName, childPath);
      <li className="flex min-h-11 items-center gap-3 px-3 py-2">
        @if (isTree) {
          <Folder aria-hidden size={16} className="shrink-0 text-muted-foreground" />;
        } @else {
          @if (isLink) {
            <Package
              aria-hidden
              size={16}
              className="shrink-0 text-muted-foreground"
            />;
          } @else {
            <FileCode2
              aria-hidden
              size={16}
              className="shrink-0 text-muted-foreground"
            />;
          }
        }
        <a
          href={href as string}
          className="min-w-0 flex-1 truncate text-[14px] font-normal text-foreground no-underline hover:underline"
          title={
            isLink
              ? (`Submodule ${entry.name} @ ${entry.oid.slice(0, 7)}` as string)
              : (entry.name as string)
          }
        >
          {entry.name as string}
        </a>
        @if (isLink) {
          <span className="shrink-0 font-mono text-[12px] text-muted-foreground">
            {entry.oid.slice(0, 7) as string}
          </span>;
        }
      </li>;
    } @empty {
      <li className="px-3 py-6 text-center text-[14px] text-muted-foreground">
        This directory is empty.
      </li>;
    }
  </ul>;
}
