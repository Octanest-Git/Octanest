import { createServerFn } from "@octanejs/tanstack-start";
import { getRequestHeader } from "@octanejs/tanstack-start/server";
import { createClient, type OctanestClient } from "@octanest/api-client";
import { resolvePublicOriginFromEnv } from "@/lib/public-origin";

/** API origin for SSR Cookie-forward RPCs — never the browser origin during SSR. */
function ssrApiOrigin(): string {
  return (
    process.env.OCTANEST_API_ORIGIN?.replace(/\/$/, "") ||
    process.env.OCTANEST_E2E_API_ORIGIN?.replace(/\/$/, "") ||
    "http://127.0.0.1:8080"
  );
}

/**
 * Cookie-forward Octanest RPC client for repo SSR loaders.
 * Forwards the incoming request Cookie only — never logs cookie values (T-06-11).
 */
function createSsrClient(cookie: string): OctanestClient {
  return createClient({
    baseUrl: ssrApiOrigin(),
    credentials: "include",
    fetch: (input, init) => {
      const headers = new Headers(init?.headers);
      if (cookie) {
        headers.set("cookie", cookie);
      }
      return fetch(input, { ...init, headers });
    },
  });
}

function incomingCookie(): string {
  return getRequestHeader("cookie") ?? "";
}

type OwnerName = { owner: string; name: string };

function ownerNameValidator(data: OwnerName): OwnerName {
  return {
    owner: String(data?.owner ?? ""),
    name: String(data?.name ?? ""),
  };
}

/** SSR: `repo.get` with Cookie forward (private repos included when session owns them). */
export const fetchRepoGet = createServerFn({ method: "GET" })
  .validator(ownerNameValidator)
  .handler(async ({ data }) => {
    const client = createSsrClient(incomingCookie());
    return client.repo.get({ owner: data.owner, name: data.name });
  });

/** SSR: `repo.tree` with Cookie forward. */
export const fetchRepoTree = createServerFn({ method: "GET" })
  .validator(
    (data: OwnerName & { ref: string; path?: string }) => ({
      owner: String(data?.owner ?? ""),
      name: String(data?.name ?? ""),
      ref: String(data?.ref ?? ""),
      path: String(data?.path ?? ""),
    }),
  )
  .handler(async ({ data }) => {
    const client = createSsrClient(incomingCookie());
    return client.repo.tree({
      owner: data.owner,
      name: data.name,
      ref: data.ref,
      path: data.path,
    });
  });

/** SSR: `repo.refs` with Cookie forward. */
export const fetchRepoRefs = createServerFn({ method: "GET" })
  .validator(ownerNameValidator)
  .handler(async ({ data }) => {
    const client = createSsrClient(incomingCookie());
    return client.repo.refs({ owner: data.owner, name: data.name });
  });

/** SSR: `repo.blob` with Cookie forward. */
export const fetchRepoBlob = createServerFn({ method: "GET" })
  .validator(
    (data: OwnerName & { ref: string; path: string }) => ({
      owner: String(data?.owner ?? ""),
      name: String(data?.name ?? ""),
      ref: String(data?.ref ?? ""),
      path: String(data?.path ?? ""),
    }),
  )
  .handler(async ({ data }) => {
    const client = createSsrClient(incomingCookie());
    return client.repo.blob({
      owner: data.owner,
      name: data.name,
      ref: data.ref,
      path: data.path,
    });
  });

/** SSR: `repo.commits` with Cookie forward. */
export const fetchRepoCommits = createServerFn({ method: "GET" })
  .validator(
    (data: OwnerName & { ref: string; skip?: number; limit?: number }) => ({
      owner: String(data?.owner ?? ""),
      name: String(data?.name ?? ""),
      ref: String(data?.ref ?? ""),
      skip: typeof data?.skip === "number" ? data.skip : 0,
      limit: typeof data?.limit === "number" ? data.limit : 30,
    }),
  )
  .handler(async ({ data }) => {
    const client = createSsrClient(incomingCookie());
    return client.repo.commits({
      owner: data.owner,
      name: data.name,
      ref: data.ref,
      skip: data.skip,
      limit: data.limit,
    });
  });

/** SSR: `repo.blame` with Cookie forward. */
export const fetchRepoBlame = createServerFn({ method: "GET" })
  .validator(
    (data: OwnerName & { ref: string; path: string }) => ({
      owner: String(data?.owner ?? ""),
      name: String(data?.name ?? ""),
      ref: String(data?.ref ?? ""),
      path: String(data?.path ?? ""),
    }),
  )
  .handler(async ({ data }) => {
    const client = createSsrClient(incomingCookie());
    return client.repo.blame({
      owner: data.owner,
      name: data.name,
      ref: data.ref,
      path: data.path,
    });
  });

/** SSR: `repo.commit` with Cookie forward. */
export const fetchRepoCommit = createServerFn({ method: "GET" })
  .validator(
    (data: OwnerName & { sha: string }) => ({
      owner: String(data?.owner ?? ""),
      name: String(data?.name ?? ""),
      sha: String(data?.sha ?? ""),
    }),
  )
  .handler(async ({ data }) => {
    const client = createSsrClient(incomingCookie());
    return client.repo.commit({
      owner: data.owner,
      name: data.name,
      sha: data.sha,
    });
  });

/** SSR: `repo.compare` with Cookie forward. */
export const fetchRepoCompare = createServerFn({ method: "GET" })
  .validator(
    (data: OwnerName & { base: string; head: string }) => ({
      owner: String(data?.owner ?? ""),
      name: String(data?.name ?? ""),
      base: String(data?.base ?? ""),
      head: String(data?.head ?? ""),
    }),
  )
  .handler(async ({ data }) => {
    const client = createSsrClient(incomingCookie());
    return client.repo.compare({
      owner: data.owner,
      name: data.name,
      base: data.base,
      head: data.head,
    });
  });

/**
 * SSR: browser-facing origin for clone URLs.
 * Prefer OCTANEST_PUBLIC_ORIGIN; fall back to forwarded Host.
 */
export const fetchPublicOrigin = createServerFn({ method: "GET" }).handler(
  async () => {
    const fromEnv = resolvePublicOriginFromEnv();
    if (fromEnv) return fromEnv;

    const host =
      getRequestHeader("x-forwarded-host")?.split(",")[0]?.trim() ||
      getRequestHeader("host")?.trim() ||
      "";
    if (!host) {
      return "http://localhost";
    }

    const protoRaw =
      getRequestHeader("x-forwarded-proto")?.split(",")[0]?.trim() ||
      (host.startsWith("localhost") || host.startsWith("127.0.0.1")
        ? "http"
        : "https");
    const proto = protoRaw === "https" ? "https" : "http";
    return `${proto}://${host}`.replace(/\/$/, "");
  },
);
