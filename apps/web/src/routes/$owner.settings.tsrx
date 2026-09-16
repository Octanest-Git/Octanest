import {
  Outlet,
  createFileRoute,
  notFound,
  redirect,
  useLoaderData,
  useLocation,
  useParams,
} from "@octanejs/tanstack-router";
import type { OrgPublic } from "@octanest/api-client";
import { OrgSettingsLayout, SettingsShell } from "@/components/settings/settings-shell";
import type { OrgSettingsNavActive } from "@/components/org/org-settings-nav";
import { fetchOrgGet, fetchOrgListMine } from "@/lib/ssr-org";
import { fetchSessionMe } from "@/lib/ssr-auth";

export type OrgSettingsLoaderData = {
  org: OrgPublic;
  canAdmin: boolean;
};

export const Route = createFileRoute("/$owner/settings")({
  component: OrgSettingsLayoutPage,
  head: ({ loaderData, params }) => {
    const data = loaderData as OrgSettingsLoaderData | undefined;
    const owner = String((params as { owner?: string }).owner ?? "");
    const name = data?.org?.display_name || data?.org?.slug || owner;
    return { meta: [{ title: `Settings · ${name} · Octanest` }] };
  },
  beforeLoad: async ({ params }) => {
    const owner = String((params as { owner?: string }).owner ?? "");
    try {
      const me = await fetchSessionMe();
      if (!me.ok) {
        throw redirect({ href: `/login?returnTo=/${owner}/settings` });
      }
    } catch (e) {
      if (e && typeof e === "object" && "href" in e) throw e;
      throw redirect({ href: `/login?returnTo=/${owner}/settings` });
    }
  },
  loader: async ({ params }): Promise<OrgSettingsLoaderData> => {
    const owner = String((params as { owner?: string }).owner ?? "");
    const got = await fetchOrgGet({ data: { slug: owner } });
    if (!got.ok) throw notFound();

    let canAdmin = false;
    const mine = await fetchOrgListMine();
    if (mine.ok) {
      const entry = mine.data.orgs.find((o) => o.slug === got.data.slug);
      canAdmin = entry?.role === "owner" || entry?.role === "admin";
    }
    if (!canAdmin) {
      throw redirect({ href: `/${got.data.slug}` });
    }
    return { org: got.data, canAdmin };
  },
});

function activeFromPath(pathname: string): OrgSettingsNavActive {
  const path = (pathname.split("?")[0] ?? "/").replace(/\/$/, "") || "/";
  if (path.endsWith("/settings/members")) return "members";
  if (path.endsWith("/settings/labels")) return "labels";
  return "general";
}

/**
 * Layout for `/{owner}/settings/*` — sidebar + Outlet.
 * General form lives on the index route so members/labels can render.
 */
export function OrgSettingsLayoutPage() @{
  const params = useParams({ from: "/$owner/settings" }) as { owner?: string };
  const data = useLoaderData({ from: "/$owner/settings" }) as OrgSettingsLoaderData | undefined;
  const pathname = useLocation({
    select: (loc) => (loc as { pathname?: string }).pathname,
  });

  const org = data?.org;
  const slug = org?.slug ?? params.owner ?? "";
  const active = activeFromPath(String(pathname ?? ""));

  @if (!org) {
    <SettingsShell bare>
      <p className="text-[16px] text-muted-foreground">Loading…</p>
    </SettingsShell>;
  } @else {
    <OrgSettingsLayout
      slug={slug}
      active={active}
      title="Organization settings"
      subtitle={"@" + slug}
      data-testid="org-settings-layout"
    >
      <Outlet />
    </OrgSettingsLayout>;
  }
}
