import {
  createFileRoute,
  Link,
  notFound,
  useLoaderData,
  useParams,
} from "@octanejs/tanstack-router";
import { buttonVariants } from "@/components/ui/button";
import {
  fetchOrgOverview,
  type OrgOverviewPayload,
} from "@/lib/ssr-org";
import { cn } from "@/lib/utils";

export type OrgOverviewLoaderData = OrgOverviewPayload;

export const Route = createFileRoute("/$owner")({
  component: OrgOverviewPage,
  head: ({ loaderData, params }) => {
    const data = loaderData as OrgOverviewLoaderData | undefined;
    const owner = String((params as { owner?: string }).owner ?? "");
    const title = data?.org
      ? `${data.org.display_name || data.org.slug} · Octanest`
      : `${owner} · Octanest`;
    return { meta: [{ title }] };
  },
  loader: async ({ params }): Promise<OrgOverviewLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const data = await fetchOrgOverview({ data: { owner } });
    if (!data) throw notFound();
    return data;
  },
});

/** Exported for org overview routes (D-ORG-06). */
export function OrgOverviewPage() @{
  const params = useParams({ from: "/$owner" }) as { owner?: string };
  const data = useLoaderData({ from: "/$owner" }) as
    | OrgOverviewLoaderData
    | undefined;

  const org = data?.org;
  const slug = org?.slug ?? params.owner ?? "";
  const display = org?.display_name?.trim() || slug;
  const memberCount = data?.memberCount;
  const repos = data?.repos ?? [];
  const canAdmin = data?.canAdmin === true;

  @if (!org) {
    <div className="mx-auto max-w-6xl px-4 py-16">
      <p className="text-[16px] text-muted-foreground">Organization not found.</p>
    </div>
  } @else {
    <div className="mx-auto max-w-6xl px-4 py-12">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="font-[family-name:var(--font-display)] text-[28px] font-semibold leading-[1.2] text-foreground">
            {display as string}
          </h1>
          <p className="mt-1 text-[16px] text-muted-foreground">
            {"@" + (slug as string)}
          </p>
          @if (memberCount != null) {
            <p className="mt-3 text-[14px] text-muted-foreground">
              {(memberCount === 1
                ? "1 member"
                : `${memberCount} members`) as string}
            </p>
          }
        </div>
        @if (canAdmin) {
          <div className="flex flex-wrap gap-2">
            <a
              href={`/${slug}/settings`}
              className={cn(buttonVariants({ variant: "secondary" }), "h-11")}
            >
              Settings
            </a>
            <a
              href={`/${slug}/settings/members`}
              className={cn(buttonVariants({ variant: "outline" }), "h-11")}
            >
              Members
            </a>
          </div>
        }
      </div>

      <section className="mt-10" aria-labelledby="org-repos-heading">
        <h2
          id="org-repos-heading"
          className="font-[family-name:var(--font-display)] text-[20px] font-semibold text-foreground"
        >
          Repositories
        </h2>
        @if (repos.length === 0) {
          <p className="mt-4 text-[16px] text-muted-foreground">
            No public repositories yet.
          </p>
        } @else {
          <ul className="mt-4 divide-y divide-border border-y border-border">
            @for (const repo of repos; key repo.id) {
              <li className="py-4">
                <Link
                  to="/$owner/$repo"
                  params={{ owner: slug, repo: repo.name }}
                  preload="intent"
                  className="text-[16px] font-medium text-foreground no-underline hover:underline"
                >
                  {repo.name as string}
                </Link>
                @if (repo.description?.trim()) {
                  <p className="mt-1 text-[14px] text-muted-foreground">
                    {repo.description as string}
                  </p>
                }
                <p className="mt-1 text-[13px] text-muted-foreground">
                  {(repo.visibility === "private" ? "Private" : "Public") as string}
                </p>
              </li>
            }
          </ul>
        }
      </section>
    </div>
  }
}
