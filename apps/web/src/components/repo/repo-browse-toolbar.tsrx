import type { RepoRefEntry } from "@oxidean/api-client";
import { CloneBox } from "@/components/repo/clone-box";
import { PathBreadcrumb } from "@/components/repo/path-breadcrumb";
import { RefSelect } from "@/components/repo/ref-select";

type RepoBrowseToolbarProps = {
  owner: string;
  repo: string;
  refName: string;
  refs: RepoRefEntry[];
  /** Directory under ref — empty on repo home. */
  path?: string;
  /** Show path crumbs after the ref select (tree/blob pages). */
  showPathCrumbs?: boolean;
  empty?: boolean;
};

/**
 * Shared Code / tree toolbar — ref select, optional path crumbs, clone/download
 * (matches repo home layout).
 */
export function RepoBrowseToolbar({
  owner,
  repo,
  refName,
  refs,
  path = "",
  showPathCrumbs = false,
  empty = false,
}: RepoBrowseToolbarProps) @{
  const mode = showPathCrumbs ? "tree" : "home";

  <div className="mb-4 flex flex-wrap items-center gap-3" data-testid="repo-browse-toolbar">
    <RefSelect
      owner={owner}
      repo={repo}
      refs={refs}
      value={refName}
      pathUnderRef={path}
      mode={mode}
    />
    @if (showPathCrumbs) {
      <div className="min-w-0 flex-1">
        <PathBreadcrumb owner={owner} repo={repo} refName={refName} path={path} />
      </div>;
    }
    <div className={showPathCrumbs ? "shrink-0 sm:ml-auto" : "ml-auto"}>
      <CloneBox owner={owner} repo={repo} refName={refName} empty={empty} />
    </div>
  </div>;
}
