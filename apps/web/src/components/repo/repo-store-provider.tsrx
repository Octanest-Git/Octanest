import { useState } from "octane";
import { RepoStoreContext, createRepoStore, type RepoLayoutStatus } from "@/lib/repo-store";
import type { RepoPublic, UserPublic } from "@octanest/api-client";

type RepoStoreProviderProps = {
  owner: string;
  repoName: string;
  status: RepoLayoutStatus;
  repo: RepoPublic | null;
  me: UserPublic | null;
  message?: string;
  publicOrigin: string;
  children?: unknown;
};

/**
 * Per-request / per-repo Zustand store via @octanejs/zustand.
 * Parent remounts with `key={owner/repo}` so SSR seed matches client hydrate.
 */
export function RepoStoreProvider({
  owner,
  repoName,
  status,
  repo,
  me,
  message,
  publicOrigin,
  children,
}: RepoStoreProviderProps) @{
  const [store] = useState(() =>
    createRepoStore({
      owner,
      repoName,
      status,
      repo,
      me,
      message,
      publicOrigin,
    }),
  );

  <RepoStoreContext.Provider value={store}>{children}</RepoStoreContext.Provider>;
}
