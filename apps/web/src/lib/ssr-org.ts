import { createServerFn } from "@octanejs/tanstack-start";
import { getRequestHeader } from "@octanejs/tanstack-start/server";
import {
  createClient,
  type OctanestClient,
  type OrgMineEntry,
  type OrgPublic,
  type RepoPublic,
} from "@octanest/api-client";

function ssrApiOrigin(): string {
  return (
    process.env.OCTANEST_API_ORIGIN?.replace(/\/$/, "") ||
    process.env.OCTANEST_E2E_API_ORIGIN?.replace(/\/$/, "") ||
    "http://127.0.0.1:8080"
  );
}

function createSsrClient(cookie: string): OctanestClient {
  return createClient({
    baseUrl: ssrApiOrigin(),
    credentials: "include",
    fetch: (input, init) => {
      const headers = new Headers(init?.headers);
      if (cookie) headers.set("cookie", cookie);
      return fetch(input, { ...init, headers });
    },
  });
}

function incomingCookie(): string {
  return getRequestHeader("cookie") ?? "";
}

export type OrgOverviewPayload = {
  org: OrgPublic;
  memberCount: number | null;
  repos: RepoPublic[];
  canAdmin: boolean;
};

/** SSR: org overview — org.get + member count + ACL-filtered repos (D-ORG-06). */
export const fetchOrgOverview = createServerFn({ method: "GET" })
  .validator((data: { owner: string }) => ({
    owner: String(data?.owner ?? ""),
  }))
  .handler(async ({ data }): Promise<OrgOverviewPayload | null> => {
    const client = createSsrClient(incomingCookie());
    const slug = data.owner.trim();
    if (!slug) return null;

    const got = await client.org.get({ slug });
    if (!got.ok) return null;

    const org = got.data;
    let memberCount: number | null = null;
    let canAdmin = false;

    const mine = await client.org.listMine();
    if (mine.ok) {
      const entry: OrgMineEntry | undefined = mine.data.orgs.find((o) => o.slug === org.slug);
      if (entry) {
        canAdmin = entry.role === "owner" || entry.role === "admin";
      }
    }

    const members = await client.org.members.list({ slug: org.slug });
    if (members.ok) {
      memberCount = members.data.members.length;
    }

    let repos: RepoPublic[] = [];
    const listed = await client.repo.listByOwner({ owner: org.slug });
    if (listed.ok) {
      repos = listed.data.repos;
    }

    return { org, memberCount, repos, canAdmin };
  });

/** SSR: org.get for settings loaders. */
export const fetchOrgGet = createServerFn({ method: "GET" })
  .validator((data: { slug: string }) => ({
    slug: String(data?.slug ?? ""),
  }))
  .handler(async ({ data }) => {
    const client = createSsrClient(incomingCookie());
    return client.org.get({ slug: data.slug.trim() });
  });

/** SSR: org.listMine for Admin+ gates. */
export const fetchOrgListMine = createServerFn({ method: "GET" }).handler(async () => {
  const client = createSsrClient(incomingCookie());
  return client.org.listMine();
});
