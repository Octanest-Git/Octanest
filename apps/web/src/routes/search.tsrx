import { createFileRoute, Link, useLoaderData } from "@octanejs/tanstack-router";
import { createServerFn } from "@octanejs/tanstack-start";
import { getRequestHeader } from "@octanejs/tanstack-start/server";
import { useState } from "octane";
import {
  createClient,
  type OrgMineEntry,
  type RepoPublic,
  type UserLookupHit,
} from "@octanest/api-client";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { SearchEntityType } from "@/lib/global-search-suggest";
import { cn } from "@/lib/utils";

function ssrApiOrigin(): string {
  return (
    process.env.OCTANEST_API_ORIGIN?.replace(/\/$/, "") ||
    process.env.OCTANEST_E2E_API_ORIGIN?.replace(/\/$/, "") ||
    "http://127.0.0.1:8080"
  );
}

const TABS: { id: SearchEntityType; label: string }[] = [
  { id: "repositories", label: "Repositories" },
  { id: "users", label: "Users" },
  { id: "organizations", label: "Organizations" },
  { id: "code", label: "Code" },
  { id: "issues", label: "Issues" },
  { id: "pulls", label: "Pull requests" },
];

function parseType(raw: string | null): SearchEntityType {
  const t = (raw ?? "repositories").toLowerCase();
  if (TABS.some((x) => x.id === t)) return t as SearchEntityType;
  return "repositories";
}

type SearchLoaderData = {
  q: string;
  type: SearchEntityType;
  repos: RepoPublic[];
  users: UserLookupHit[];
  orgs: OrgMineEntry[];
  signedIn: boolean;
};

async function loadAccessibleRepos(
  client: ReturnType<typeof createClient>,
  q: string,
  signedIn: boolean,
): Promise<RepoPublic[]> {
  const [exploreRes, mineRes] = await Promise.all([
    client.repo.explore({ q: q || null, offset: 0, limit: 40 }),
    signedIn ? client.repo.listMine() : Promise.resolve(null),
  ]);
  const byId = new Map<string, RepoPublic>();
  if (exploreRes.ok) {
    for (const r of exploreRes.data.repos) byId.set(r.id, r);
  }
  if (mineRes?.ok) {
    const needle = q.toLowerCase();
    for (const r of mineRes.data.repos) {
      if (!needle) {
        byId.set(r.id, r);
        continue;
      }
      const key = `${r.owner_username}/${r.name} ${r.description ?? ""}`.toLowerCase();
      if (key.includes(needle)) byId.set(r.id, r);
    }
  }
  return [...byId.values()];
}

const fetchSearch = createServerFn({ method: "GET" })
  .validator((data: { q?: string; type?: string }) => ({
    q: data?.q ? String(data.q) : "",
    type: data?.type ? String(data.type) : "repositories",
  }))
  .handler(async ({ data }): Promise<SearchLoaderData> => {
    const cookie = getRequestHeader("cookie") ?? "";
    const client = createClient({
      baseUrl: ssrApiOrigin(),
      credentials: "include",
      fetch: (input, init) => {
        const headers = new Headers(init?.headers);
        if (cookie) headers.set("cookie", cookie);
        return fetch(input, { ...init, headers });
      },
    });

    const q = data.q.trim();
    const type = parseType(data.type);
    const me = await client.auth.me();
    const signedIn = me.ok;

    let repos: RepoPublic[] = [];
    let users: UserLookupHit[] = [];
    let orgs: OrgMineEntry[] = [];

    if (type === "repositories" || type === "code" || type === "issues" || type === "pulls") {
      repos = await loadAccessibleRepos(client, q, signedIn);
    }

    if (type === "users" && signedIn && q.length >= 2 && !q.includes("@")) {
      const res = await client.user.lookup({ prefix: q });
      users = res.ok ? res.data.users : [];
    }

    if (type === "organizations" && signedIn) {
      const res = await client.org.listMine();
      if (res.ok) {
        orgs = q
          ? res.data.orgs.filter((o) =>
              `${o.slug} ${o.display_name ?? ""}`.toLowerCase().includes(q.toLowerCase()),
            )
          : res.data.orgs;
      }
    }

    return { q, type, repos, users, orgs, signedIn };
  });

export const Route = createFileRoute("/search")({
  component: SearchPage,
  head: ({ loaderData }) => {
    const q = (loaderData as SearchLoaderData | undefined)?.q?.trim();
    return {
      meta: [{ title: q ? `Search · ${q} · Octanest` : "Search · Octanest" }],
    };
  },
  loader: async ({ location }): Promise<SearchLoaderData> => {
    const params = new URLSearchParams(location.searchStr.replace(/^\?/, ""));
    return fetchSearch({
      data: {
        q: params.get("q") ?? "",
        type: params.get("type") ?? "repositories",
      },
    });
  },
});

function tabHref(q: string, type: SearchEntityType): string {
  const p = new URLSearchParams();
  if (q.trim()) p.set("q", q.trim());
  p.set("type", type);
  return `/search?${p.toString()}`;
}

function tabClass(active: boolean): string {
  return cn(
    "border-b-2 px-1 pb-2 text-[14px] no-underline",
    active
      ? "border-primary font-semibold text-foreground"
      : "border-transparent font-normal text-muted-foreground hover:text-foreground",
  );
}

/** Sitewide search results — repos (incl. private you can access), users, orgs. */
export function SearchPage() @{
  const data = useLoaderData({ from: "/search" }) as SearchLoaderData | undefined;
  const q0 = data?.q ?? "";
  const type = data?.type ?? "repositories";
  const repos = data?.repos ?? [];
  const users = data?.users ?? [];
  const orgs = data?.orgs ?? [];
  const signedIn = data?.signedIn === true;
  const [q, setQ] = useState(q0);

  const scoped = type === "code" || type === "issues" || type === "pulls";
  const scopedLabel = type === "code" ? "code" : type === "issues" ? "issues" : "pull requests";

  <div className="mx-auto max-w-5xl px-4 py-10 sm:px-6" data-testid="search-page">
    <h1 className="font-[family-name:var(--font-display)] text-[28px] font-semibold text-foreground">
      Search
    </h1>
    <p className="mt-2 text-[15px] text-muted-foreground">
      Find repositories you can access (including private), users, and organizations.
    </p>

    <form
      method="get"
      action="/search"
      className="mt-6 flex max-w-xl flex-wrap gap-2"
      onSubmit={(e) => {
        e.preventDefault();
        window.location.assign(tabHref(q, type));
      }}
    >
      <Input
        name="q"
        value={q as string}
        onInput={(e) => setQ((e.currentTarget as HTMLInputElement).value)}
        placeholder="Search Octanest…"
        className="h-11 flex-1"
        aria-label="Search query"
        data-testid="search-page-input"
      />
      <input type="hidden" name="type" value={type as string} />
      <Button type="submit" className="h-11">
        Search
      </Button>
    </form>

    <nav
      className="mt-8 flex flex-wrap gap-4 border-b border-border"
      aria-label="Search categories"
    >
      @for (const tab of TABS; key tab.id) {
        <a
          href={tabHref(q0, tab.id) as string}
          className={tabClass(type === tab.id) as string}
          aria-current={type === tab.id ? "page" : undefined}
        >
          {tab.label as string}
        </a>;
      }
    </nav>

    <div className="mt-8">
      @if (scoped) {
        <div
          className="rounded-md border border-border bg-card p-5"
          data-testid="search-scoped-hint"
        >
          <h2 className="text-[16px] font-semibold text-foreground">
            {
              (type === "code"
                ? "Code search"
                : type === "issues"
                  ? "Issue search"
                  : "Pull request search") as string
            }
          </h2>
          <p className="mt-2 text-[14px] leading-relaxed text-muted-foreground">
            {scopedLabel as string} are searched inside a repository. Open a repository below,
            then use its Search page — same pattern as GitHub scoped search.
          </p>
          @if (repos.length === 0) {
            <p className="mt-4 text-[14px] text-muted-foreground">
              {q0.trim()
                ? "No matching repositories."
                : "Enter a query to list repositories to search in."}
            </p>;
          } @else {
            <ul className="mt-4 divide-y divide-border border-y border-border">
              @for (const repo of repos; key repo.id) {
                <li className="flex flex-wrap items-center justify-between gap-2 py-3">
                  <a
                    href={"/" + repo.owner_username + "/" + repo.name}
                    className="font-medium text-primary no-underline hover:underline"
                  >
                    {(repo.owner_username + "/" + repo.name) as string}
                  </a>
                  <a
                    href={"/" + repo.owner_username + "/" + repo.name + "/search"}
                    className="text-[13px] text-muted-foreground no-underline hover:text-foreground hover:underline"
                  >
                    Search in repository
                  </a>
                </li>;
              }
            </ul>;
          }
        </div>;
      } @else if (type === "repositories") {
        @if (repos.length === 0) {
          <p className="text-[15px] text-muted-foreground">No repositories found.</p>;
        } @else {
          <ul className="divide-y divide-border border-y border-border">
            @for (const repo of repos; key repo.id) {
              <li className="py-4">
                <a
                  href={"/" + repo.owner_username + "/" + repo.name}
                  className="text-[16px] font-semibold text-primary no-underline hover:underline"
                >
                  {(repo.owner_username + "/" + repo.name) as string}
                </a>
                <div className="mt-1 flex flex-wrap items-center gap-2">
                  <Badge variant="outline">
                    {(repo.visibility === "private" ? "Private" : "Public") as string}
                  </Badge>
                  @if (repo.description?.trim()) {
                    <span className="text-[14px] text-muted-foreground">
                      {repo.description as string}
                    </span>;
                  }
                </div>
              </li>;
            }
          </ul>;
        }
      } @else if (type === "users") {
        @if (!signedIn) {
          <p className="text-[15px] text-muted-foreground">
            <a href="/login?returnTo=/search" className="text-primary hover:underline">
              Sign in
            </a>{" "}
            to search users.
          </p>;
        } @else if (q0.trim().length < 2) {
          <p className="text-[15px] text-muted-foreground">
            Type at least 2 characters to search users.
          </p>;
        } @else if (users.length === 0) {
          <p className="text-[15px] text-muted-foreground">No users found.</p>;
        } @else {
          <ul className="divide-y divide-border border-y border-border">
            @for (const u of users; key u.username) {
              <li className="py-3">
                <a
                  href={"/" + u.username}
                  className="text-[16px] font-semibold text-primary no-underline hover:underline"
                >
                  {"@" + (u.username as string)}
                </a>
                @if (u.display_name) {
                  <span className="ml-2 text-[14px] text-muted-foreground">
                    {u.display_name as string}
                  </span>;
                }
              </li>;
            }
          </ul>;
        }
      } @else {
        @if (!signedIn) {
          <p className="text-[15px] text-muted-foreground">
            <a href="/login?returnTo=/search" className="text-primary hover:underline">
              Sign in
            </a>{" "}
            to search organizations you belong to.
          </p>;
        } @else if (orgs.length === 0) {
          <p className="text-[15px] text-muted-foreground">No organizations found.</p>;
        } @else {
          <ul className="divide-y divide-border border-y border-border">
            @for (const o of orgs; key o.slug) {
              <li className="py-3">
                <Link
                  to="/$owner"
                  params={{ owner: o.slug }}
                  preload="intent"
                  className="text-[16px] font-semibold text-primary no-underline hover:underline"
                >
                  {o.slug as string}
                </Link>
                @if (o.display_name) {
                  <span className="ml-2 text-[14px] text-muted-foreground">
                    {o.display_name as string}
                  </span>;
                }
              </li>;
            }
          </ul>;
        }
      }
    </div>
  </div>;
}
