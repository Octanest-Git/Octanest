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
import { Building2, BookMarked, Search, Users } from "@octanejs/lucide";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { Input } from "@/components/ui/input";
import { List, ListRow } from "@/components/ui/list";
import { PageHeader } from "@/components/ui/page-header";
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
    <PageHeader
      icon={<Search aria-hidden size={22} />}
      title="Search"
      description="Find repositories you can access (including private), users, and organizations."
      divider={false}
    />

    <form
      method="get"
      action="/search"
      className="mt-6 flex max-w-xl gap-2"
      onSubmit={(e) => {
        e.preventDefault();
        window.location.assign(tabHref(q, type));
      }}
    >
      <div className="relative flex-1">
        <Search
          aria-hidden
          size={16}
          className="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-muted-foreground"
        />
        <Input
          name="q"
          value={q as string}
          onInput={(e) => setQ((e.currentTarget as HTMLInputElement).value)}
          placeholder="Search Octanest…"
          className="pl-8"
          aria-label="Search query"
          data-testid="search-page-input"
        />
      </div>
      <input type="hidden" name="type" value={type as string} />
      <Button type="submit">Search</Button>
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
            <List className="mt-4">
              @for (const repo of repos; key repo.id) {
                <ListRow className="justify-between gap-2">
                  <a
                    href={"/" + repo.owner_username + "/" + repo.name}
                    className="flex min-w-0 items-center gap-2 font-medium text-primary no-underline hover:underline"
                  >
                    <BookMarked
                      aria-hidden
                      size={16}
                      className="shrink-0 text-muted-foreground"
                    />
                    <span className="truncate">
                      {(repo.owner_username + "/" + repo.name) as string}
                    </span>
                  </a>
                  <a
                    href={"/" + repo.owner_username + "/" + repo.name + "/search"}
                    className="shrink-0 text-[13px] text-muted-foreground no-underline hover:text-foreground hover:underline"
                  >
                    Search in repository
                  </a>
                </ListRow>;
              }
            </List>;
          }
        </div>;
      } @else if (type === "repositories") {
        @if (repos.length === 0) {
          <EmptyState
            icon={<BookMarked aria-hidden size={22} />}
            title="No repositories found"
            description={
              q0.trim()
                ? "No repositories match your search. Try different keywords."
                : "Search for repositories by name or description."
            }
          />;
        } @else {
          <List>
            @for (const repo of repos; key repo.id) {
              <ListRow className="items-start gap-3 py-4">
                <BookMarked
                  aria-hidden
                  size={18}
                  className="mt-0.5 shrink-0 text-muted-foreground"
                />
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <a
                      href={"/" + repo.owner_username + "/" + repo.name}
                      className="text-[16px] font-semibold text-primary no-underline hover:underline"
                    >
                      {(repo.owner_username + "/" + repo.name) as string}
                    </a>
                    <Badge
                      variant={repo.visibility === "private" ? "outline" : "success"}
                    >
                      {(repo.visibility === "private" ? "Private" : "Public") as string}
                    </Badge>
                  </div>
                  @if (repo.description?.trim()) {
                    <p className="mt-1 text-[14px] text-muted-foreground">
                      {repo.description as string}
                    </p>;
                  }
                </div>
              </ListRow>;
            }
          </List>;
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
          <EmptyState
            icon={<Users aria-hidden size={22} />}
            title="No users found"
            description="No users match your search. Try a different username."
          />;
        } @else {
          <List>
            @for (const u of users; key u.username) {
              <ListRow className="gap-2">
                <Users
                  aria-hidden
                  size={16}
                  className="shrink-0 text-muted-foreground"
                />
                <a
                  href={"/" + u.username}
                  className="text-[15px] font-semibold text-primary no-underline hover:underline"
                >
                  {"@" + (u.username as string)}
                </a>
                @if (u.display_name) {
                  <span className="text-[14px] text-muted-foreground">
                    {u.display_name as string}
                  </span>;
                }
              </ListRow>;
            }
          </List>;
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
          <EmptyState
            icon={<Building2 aria-hidden size={22} />}
            title="No organizations found"
            description="You don't belong to any matching organizations."
          />;
        } @else {
          <List>
            @for (const o of orgs; key o.slug) {
              <ListRow className="gap-2">
                <Building2
                  aria-hidden
                  size={16}
                  className="shrink-0 text-muted-foreground"
                />
                <Link
                  to="/$owner"
                  params={{ owner: o.slug }}
                  preload="intent"
                  className="text-[15px] font-semibold text-primary no-underline hover:underline"
                >
                  {o.slug as string}
                </Link>
                @if (o.display_name) {
                  <span className="text-[14px] text-muted-foreground">
                    {o.display_name as string}
                  </span>;
                }
              </ListRow>;
            }
          </List>;
        }
      }
    </div>
  </div>;
}
