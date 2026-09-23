/**
 * Unit-only bun:test preload: stub Octane packages that ship `.tsrx` entrypoints
 * Bun cannot parse without the Vite Octane plugin. Integration/browser suites use
 * happy-dom / WebView instead.
 */
import { mock } from "bun:test";

mock.module("@octanejs/tanstack-start", () => ({
  createServerFn: () => ({
    handler: (fn: unknown) => fn,
  }),
}));

mock.module("@octanejs/tanstack-start/server", () => ({
  getRequestHeader: () => undefined,
}));

mock.module("@octanejs/tanstack-router", () => ({
  Link: () => null,
  useNavigate: () => () => {},
  useRouter: () => ({}),
  useParams: () => ({}),
  useSearch: () => ({}),
  createFileRoute: () => () => ({}),
  createRootRoute: () => () => ({}),
  createRouter: () => ({}),
  Outlet: () => null,
  redirect: (o: unknown) => o,
}));

mock.module("@octanejs/tanstack-query", () => {
  class QueryClient {
    #data = new Map<string, unknown>();
    #invalidated = new Set<string>();
    #key(key: unknown) {
      return JSON.stringify(key);
    }
    getQueryData(key: unknown) {
      return this.#data.get(this.#key(key));
    }
    setQueryData(key: unknown, value: unknown) {
      const k = this.#key(key);
      const next =
        typeof value === "function"
          ? (value as (prev: unknown) => unknown)(this.#data.get(k))
          : value;
      this.#data.set(k, next);
    }
    getQueryState(key: unknown) {
      const k = this.#key(key);
      if (!this.#data.has(k) && !this.#invalidated.has(k)) {
        return undefined;
      }
      return { data: this.#data.get(k), isInvalidated: this.#invalidated.has(k) };
    }
    clear() {
      this.#data.clear();
      this.#invalidated.clear();
    }
    invalidateQueries(filters?: { queryKey?: unknown }) {
      if (filters?.queryKey) {
        this.#invalidated.add(this.#key(filters.queryKey));
      }
      return Promise.resolve();
    }
  }
  return {
    QueryClient,
    queryOptions: (o: unknown) => o,
    QueryClientProvider: ({ children }: { children?: unknown }) => children,
    useQuery: () => ({ data: undefined, isLoading: false }),
    useMutation: () => ({ mutate: () => {}, mutateAsync: async () => {} }),
  };
});
