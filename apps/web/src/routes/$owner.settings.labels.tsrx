import {
  createFileRoute,
  notFound,
  redirect,
  useLoaderData,
  useParams,
} from "@octanejs/tanstack-router";
import { useForm } from "@octanejs/tanstack-form";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import type { LabelPublic, OrgPublic } from "@octanest/api-client";
import { useState } from "octane";
import { AuthErrorBanner } from "@/components/auth-shell";
import { OrgSettingsNav } from "@/components/org/org-settings-nav";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { apiClient } from "@/lib/api-client";
import { fetchOrgGet, fetchOrgListMine } from "@/lib/ssr-org";
import { fetchSessionMe } from "@/lib/ssr-auth";

export type OrgLabelsLoaderData = {
  org: OrgPublic;
  canAdmin: boolean;
};

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const FORBIDDEN = "You need organization admin access to manage labels.";

function labelsQueryKey(slug: string) {
  return ["label", "listForOrg", slug] as const;
}

function swatchStyle(color: string): { backgroundColor: string } {
  const hex = color.startsWith("#") ? color : `#${color}`;
  return { backgroundColor: hex };
}

export const Route = createFileRoute("/$owner/settings/labels")({
  component: OrgLabelsPage,
  head: ({ loaderData, params }) => {
    const data = loaderData as OrgLabelsLoaderData | undefined;
    const owner = String((params as { owner?: string }).owner ?? "");
    const name = data?.org?.display_name || data?.org?.slug || owner;
    return { meta: [{ title: `Labels · ${name}` }] };
  },
  beforeLoad: async ({ params }) => {
    const owner = String((params as { owner?: string }).owner ?? "");
    try {
      const me = await fetchSessionMe();
      if (!me.ok) {
        throw redirect({
          href: `/login?returnTo=/${owner}/settings/labels`,
        });
      }
    } catch (e) {
      if (e && typeof e === "object" && "href" in e) throw e;
      throw redirect({
        href: `/login?returnTo=/${owner}/settings/labels`,
      });
    }
  },
  loader: async ({ params }): Promise<OrgLabelsLoaderData> => {
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

/** Org default label catalog — Admin only (D-ISS-05 / D-ISS-07). */
export function OrgLabelsPage() @{
  const params = useParams({ strict: false }) as { owner?: string };
  const data = useLoaderData({ from: "/$owner/settings/labels" }) as
    | OrgLabelsLoaderData
    | undefined;
  const org = data?.org;
  const slug = org?.slug ?? params.owner ?? "";
  const queryClient = useQueryClient();

  const labelsQ = useQuery({
    queryKey: labelsQueryKey(slug),
    queryFn: async () => {
      const res = await apiClient.label.listForOrg({ slug });
      if (!res.ok) throw new Error(res.error.message || NETWORK_ERROR);
      return res.data.labels;
    },
    enabled: !!slug,
  });

  const [formError, setFormError] = useState("");
  const [pending, setPending] = useState(false);
  const [rowError, setRowError] = useState("");

  const form = useForm({
    defaultValues: {
      name: "",
      color: "0366d6",
      description: "",
    },
    onSubmit: async ({ value }) => {
      setFormError("");
      const trimmed = value.name.trim();
      if (!trimmed) {
        setFormError("Name is required.");
        return;
      }
      setPending(true);
      try {
        const res = await apiClient.label.create({
          scope: "org",
          owner: slug,
          name: trimmed,
          color: value.color,
          description: value.description.trim() || null,
        });
        if (!res.ok) {
          if (res.error.code === "org.forbidden") {
            setFormError(FORBIDDEN);
          } else {
            setFormError(res.error.message || NETWORK_ERROR);
          }
          setPending(false);
          return;
        }
        form.reset();
        setPending(false);
        await refresh();
      } catch {
        setFormError(NETWORK_ERROR);
        setPending(false);
      }
    },
  });

  const labels: LabelPublic[] = labelsQ.data ?? [];

  async function refresh() {
    await queryClient.invalidateQueries({ queryKey: labelsQueryKey(slug) });
  }

  async function deleteLabel(lab: LabelPublic) {
    setRowError("");
    try {
      const res = await apiClient.label.delete({
        id: lab.id,
        owner: slug,
      });
      if (!res.ok) {
        setRowError(res.error.message || NETWORK_ERROR);
        return;
      }
      await refresh();
    } catch {
      setRowError(NETWORK_ERROR);
    }
  }

  <div className="mx-auto max-w-3xl px-4 py-8">
    <h1 className="mb-2 font-[family-name:var(--font-display)] text-[24px] font-semibold text-foreground">
      Organization settings
    </h1>
    <OrgSettingsNav slug={slug} active="labels" />

    <h2 className="mb-1 text-[18px] font-semibold text-foreground">Labels</h2>
    <p className="mb-6 text-[14px] text-muted-foreground">
      Default labels inherited by organization repositories. Repos can hide or add local labels.
    </p>

    @if (formError) {
      <div className="mb-4">
        <AuthErrorBanner message={formError} />
      </div>;
    }
    @if (rowError) {
      <div className="mb-4">
        <AuthErrorBanner message={rowError} />
      </div>;
    }

    <form
      method="post"
      action="#"
      className="mb-8 flex flex-col gap-3 rounded-md border border-border p-4"
      onSubmit={(e) => {
        e.preventDefault();
        void form.handleSubmit();
      }}
    >
      <p className="text-[14px] font-semibold text-foreground">New label</p>
      <div className="grid gap-3 sm:grid-cols-2">
        <form.Field name="name">
          {(field) => (
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="org-label-name">Name</Label>
              <Input
                id="org-label-name"
                value={field.state.value}
                onBlur={field.handleBlur}
                onInput={(e) => field.handleChange((e.target as HTMLInputElement).value)}
                disabled={pending}
              />
            </div>
          )}
        </form.Field>
        <form.Field name="color">
          {(field) => (
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="org-label-color">Color (hex)</Label>
              <Input
                id="org-label-color"
                value={field.state.value}
                onBlur={field.handleBlur}
                onInput={(e) => field.handleChange((e.target as HTMLInputElement).value)}
                disabled={pending}
              />
            </div>
          )}
        </form.Field>
      </div>
      <form.Field name="description">
        {(field) => (
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="org-label-desc">Description</Label>
            <Input
              id="org-label-desc"
              value={field.state.value}
              onBlur={field.handleBlur}
              onInput={(e) => field.handleChange((e.target as HTMLInputElement).value)}
              disabled={pending}
            />
          </div>
        )}
      </form.Field>
      <div>
        <Button type="submit" disabled={pending}>
          Create label
        </Button>
      </div>
    </form>

    @if (labelsQ.isPending) {
      <p className="text-[16px] text-muted-foreground">Loading labels…</p>;
    } @else {
      @if (labels.length === 0) {
        <p className="text-[16px] text-muted-foreground">No org labels yet.</p>;
      } @else {
        <ul className="divide-y divide-border border-y border-border">
          @for (const lab of labels; key lab.id) {
            <li className="flex flex-wrap items-center justify-between gap-3 py-3">
              <div className="flex items-center gap-2">
                <span
                  className="inline-block size-3 rounded-sm border border-border"
                  style={swatchStyle(lab.color)}
                  aria-hidden
                />
                <span className="text-[15px] font-medium text-foreground">
                  {lab.name as string}
                </span>
                @if (lab.description) {
                  <span className="text-[13px] text-muted-foreground">
                    {lab.description as string}
                  </span>;
                }
              </div>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => deleteLabel(lab)}
              >
                Delete
              </Button>
            </li>;
          }
        </ul>;
      }
    }
  </div>;
}

export default OrgLabelsPage;
