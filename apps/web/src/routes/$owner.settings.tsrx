import {
  createFileRoute,
  notFound,
  redirect,
  useLoaderData,
  useParams,
} from "@octanejs/tanstack-router";
import type { MemberBasePermission, OrgPublic } from "@octanest/api-client";
import { Check, ChevronDown } from "@octanejs/lucide";
import { useForm } from "@octanejs/tanstack-form";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  SelectIcon,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectList,
  SelectPopup,
  SelectPortal,
  SelectPositioner,
  SelectRoot,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { apiClient } from "@/lib/api-client";
import {
  fetchOrgGet,
  fetchOrgListMine,
} from "@/lib/ssr-org";
import { fetchSessionMe } from "@/lib/ssr-auth";
import { toastError, toastSuccess } from "@/lib/toast";
import { OrgSettingsNav } from "@/components/org/org-settings-nav";
import { cn } from "@/lib/utils";

export type OrgSettingsLoaderData = {
  org: OrgPublic;
  canAdmin: boolean;
};

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";
const FORBIDDEN = "You need organization admin access to manage settings.";
const MEMBER_BASE_HELPER =
  "Applies to Members on private organization repositories. Owners and Admins always have admin access. Public repositories remain readable.";

const MEMBER_BASE_OPTIONS: {
  value: MemberBasePermission;
  label: string;
}[] = [
  { value: "none", label: "None" },
  { value: "read", label: "Read" },
  { value: "write", label: "Write" },
];

export const Route = createFileRoute("/$owner/settings")({
  component: OrgSettingsPage,
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
      canAdmin =
        entry?.role === "owner" || entry?.role === "admin";
    }
    if (!canAdmin) {
      throw redirect({ href: `/${got.data.slug}` });
    }
    return { org: got.data, canAdmin };
  },
});

/** Exported for member_base Wave 0 tests (D-ORG-02b). */
export function OrgSettingsPage() @{
  const params = useParams({ from: "/$owner/settings" }) as { owner?: string };
  const data = useLoaderData({ from: "/$owner/settings" }) as
    | OrgSettingsLoaderData
    | undefined;

  const org = data?.org;
  const slug = org?.slug ?? params.owner ?? "";

  const [pending, setPending] = useState(false);

  const form = useForm({
    defaultValues: {
      display_name: org?.display_name ?? "",
      member_base: (org?.member_base_permission ?? "none") as MemberBasePermission,
    },
    onSubmit: async ({ value }) => {
      setPending(true);
      try {
        const res = await apiClient.org.updateSettings({
          slug,
          display_name: value.display_name.trim() || null,
          member_base_permission: value.member_base,
        });
        if (!res.ok) {
          if (res.error.code === "org.forbidden") {
            toastError(FORBIDDEN);
          } else {
            toastError(res.error.message || NETWORK_ERROR);
          }
          setPending(false);
          return;
        }
        form.setFieldValue("display_name", res.data.display_name ?? "");
        form.setFieldValue("member_base", res.data.member_base_permission);
        toastSuccess("Settings saved.");
        setPending(false);
      } catch {
        toastError(NETWORK_ERROR);
        setPending(false);
      }
    },
  });

  @if (!org) {
    <div className="mx-auto max-w-2xl px-4 py-16">
      <p className="text-[16px] text-muted-foreground">Loading…</p>
    </div>
  } @else {
    <div className="mx-auto max-w-2xl px-4 py-12">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
        Organization settings
      </h1>
      <p className="mt-1 text-[14px] text-muted-foreground">
        {"@" + (slug as string)}
      </p>

      <div className="mt-6">
        <OrgSettingsNav slug={slug} active="general" />
      </div>

      <form
        className="mt-2 flex flex-col gap-6"
        method="post"
        action="#"
        onSubmit={(e) => {
          e.preventDefault();
          void form.handleSubmit();
        }}
      >
        <form.Field name="display_name">
          {(field) => (
            <div className="flex flex-col gap-2">
              <Label htmlFor="org-display-name">Display name</Label>
              <Input
                id="org-display-name"
                name="display_name"
                autoComplete="organization"
                value={field.state.value}
                onBlur={field.handleBlur}
                onInput={(ev) =>
                  field.handleChange((ev.currentTarget as HTMLInputElement).value)
                }
              />
            </div>
          )}
        </form.Field>

        <form.Subscribe selector={(s) => s.values.member_base}>
          {(memberBase) => {
            const baseLabel =
              MEMBER_BASE_OPTIONS.find((o) => o.value === memberBase)?.label ??
              "None";
            return (
              <div className="flex flex-col gap-2">
                <Label htmlFor="org-member-base">Base permission for Members</Label>
                <SelectRoot
                  value={memberBase}
                  onValueChange={(next) => {
                    if (typeof next === "string") {
                      form.setFieldValue("member_base", next as MemberBasePermission);
                    }
                  }}
                >
                  <SelectTrigger
                    id="org-member-base"
                    className="w-full min-w-[12rem]"
                    aria-label="Base permission for Members"
                  >
                    <SelectValue>{baseLabel as string}</SelectValue>
                    <SelectIcon>
                      <ChevronDown aria-hidden size={16} />
                    </SelectIcon>
                  </SelectTrigger>
                  <SelectPortal>
                    <SelectPositioner sideOffset={4}>
                      <SelectPopup>
                        <SelectList>
                          @for (const opt of MEMBER_BASE_OPTIONS; key opt.value) {
                            <SelectItem value={opt.value}>
                              <SelectItemIndicator>
                                <Check
                                  aria-hidden
                                  size={16}
                                  className="text-primary"
                                />
                              </SelectItemIndicator>
                              <SelectItemText>{opt.label as string}</SelectItemText>
                            </SelectItem>
                          }
                        </SelectList>
                      </SelectPopup>
                    </SelectPositioner>
                  </SelectPortal>
                </SelectRoot>
                <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
                  {MEMBER_BASE_HELPER}
                </p>
              </div>
            );
          }}
        </form.Subscribe>

        <Button
          type="submit"
          disabled={pending}
          className={cn("h-11")}
        >
          @if (pending) {
            <>Saving…</>
          } @else {
            <>Save settings</>
          }
        </Button>
      </form>
    </div>
  }
}
