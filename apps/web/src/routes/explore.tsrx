import { createFileRoute, Link, useLoaderData } from "@octanejs/tanstack-router";
import { createServerFn } from "@octanejs/tanstack-start";
import { getRequestHeader } from "@octanejs/tanstack-start/server";
import { useState } from "octane";
import { createClient, type RepoPublic } from "@octanest/api-client";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

function ssrApiOrigin(): string {
  return (
    process.env.OCTANEST_API_ORIGIN?.replace(/\/$/, "") ||
    process.env.OCTANEST_E2E_API_ORIGIN?.replace(/\/$/, "") ||
    "http://127.0.0.1:8080"
  );
}

const fetchExplore = createServerFn({ method: "GET" })
  .validator((data: { q?: string }) => ({
    q: data?.q ? String(data.q) : "",
  }))
  .handler(async ({ data }): Promise<{ repos: RepoPublic[]; q: string }> => {
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
    const res = await client.repo.explore({
      q: q || null,
      offset: 0,
      limit: 40,
    });
    return { repos: res.ok ? res.data.repos : [], q };
  });

export type ExploreLoaderData = { repos: RepoPublic[]; q: string };

export const Route = createFileRoute("/explore")({
  component: ExplorePage,
  head: () => ({ meta: [{ title: "Explore · Octanest" }] }),
  loader: async ({ location }): Promise<ExploreLoaderData> => {
    const q = new URLSearchParams(location.searchStr.replace(/^\?/, "")).get("q") ?? "";
    return fetchExplore({ data: { q } });
  },
});

export function ExplorePage() @{
  const data = useLoaderData({ from: "/explore" }) as ExploreLoaderData | undefined;
  const repos = data?.repos ?? [];
  const [q, setQ] = useState(data?.q ?? "");

  <div className="relative min-h-[70vh] overflow-hidden">
    <div
      className="pointer-events-none absolute inset-0 opacity-40"
      style={{
        background:
          "radial-gradient(ellipse 80% 50% at 20% 0%, color-mix(in oklab, var(--primary) 18%, transparent), transparent), radial-gradient(ellipse 60% 40% at 90% 10%, color-mix(in oklab, var(--chart-2, var(--primary)) 12%, transparent), transparent)",
      }}
    />
    <div className="relative mx-auto max-w-6xl px-4 py-12">
      <h1 className="font-[family-name:var(--font-display)] text-[36px] font-semibold leading-[1.1] text-foreground">
        Explore
      </h1>
      <p className="mt-2 max-w-xl text-[16px] text-muted-foreground">
        Discover public repositories on this instance, sorted by stars.
      </p>

      <form
        method="get"
        action="/explore"
        className="mt-8 flex max-w-lg flex-wrap gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          const next = q.trim();
          window.location.assign(next ? `/explore?q=${encodeURIComponent(next)}` : "/explore");
        }}
      >
        <Input
          name="q"
          value={q as string}
          onInput={(e) => setQ((e.currentTarget as HTMLInputElement).value)}
          placeholder="Filter by name or description"
          className="h-11 flex-1"
          aria-label="Search explore"
        />
        <Button type="submit" className="h-11">
          Search
        </Button>
      </form>

      @if (repos.length === 0) {
        <p className="mt-10 text-[16px] text-muted-foreground">
          No public repositories found.
        </p>;
      } @else {
        <ul className="mt-10 divide-y divide-border border-y border-border">
          @for (const repo of repos; key repo.id) {
            <li className="py-5">
              <Link
                to="/$owner/$repo"
                params={{ owner: repo.owner_username, repo: repo.name }}
                preload="intent"
                className="font-[family-name:var(--font-display)] text-[18px] font-semibold text-foreground no-underline hover:underline"
              >
                {(repo.owner_username + "/" + repo.name) as string}
              </Link>
              @if (repo.description?.trim()) {
                <p className="mt-1 text-[15px] text-muted-foreground">
                  {repo.description as string}
                </p>;
              }
              <p className="mt-2 text-[13px] text-muted-foreground">
                {(String(repo.star_count ?? 0) + " stars") as string}
              </p>
            </li>;
          }
        </ul>;
      }
    </div>
  </div>;
}
