import { createStore, useStore } from "@octanejs/zustand";
import { createContext, useContext } from "octane";
import type { RepoPublic, UserPublic } from "@octanest/api-client";

export type RepoLayoutStatus = "ok" | "not_found" | "error";

export type RepoLayoutLoaderData = {
  owner: string;
  repoName: string;
  status: RepoLayoutStatus;
  repo: RepoPublic | null;
  me: UserPublic | null;
  message: string;
  /** Browser-facing origin for clone URLs (SSR’d). */
  publicOrigin: string;
};

export type RepoStoreState = {
  owner: string;
  repoName: string;
  status: RepoLayoutStatus;
  repo: RepoPublic | null;
  me: UserPublic | null;
  message: string;
  publicOrigin: string;
  setRepo: (repo: RepoPublic) => void;
  setMe: (me: UserPublic | null) => void;
};

export type RepoStoreApi = ReturnType<typeof createRepoStore>;

export function createRepoStore(init: {
  owner: string;
  repoName: string;
  status: RepoLayoutStatus;
  repo: RepoPublic | null;
  me: UserPublic | null;
  message?: string;
  publicOrigin?: string;
}) {
  return createStore<RepoStoreState>((set) => ({
    owner: init.owner,
    repoName: init.repoName,
    status: init.status,
    repo: init.repo,
    me: init.me,
    message: init.message ?? "",
    publicOrigin: init.publicOrigin ?? "",
    setRepo: (repo) => set({ repo, status: "ok" }),
    setMe: (me) => set({ me }),
  }));
}

export const RepoStoreContext = createContext<RepoStoreApi | null>(null);

/** Used when a page is rendered outside the layout (unit tests). */
const fallbackRepoStore = createRepoStore({
  owner: "",
  repoName: "",
  status: "error",
  repo: null,
  me: null,
  publicOrigin: "",
});

export function useRepoStore<T>(selector: (s: RepoStoreState) => T): T {
  const store = useContext(RepoStoreContext) ?? fallbackRepoStore;
  return useStore(store, selector);
}

export function useRepoStoreApi(): RepoStoreApi {
  const store = useContext(RepoStoreContext);
  if (!store) {
    throw new Error("useRepoStoreApi must be used within RepoStoreProvider");
  }
  return store;
}
