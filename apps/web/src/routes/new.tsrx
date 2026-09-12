import {
  createFileRoute,
  redirect,
  useLoaderData,
} from "@octanejs/tanstack-router";
import { Check, ChevronDown } from "@octanejs/lucide";
import type {
  RepoCreateDefaults,
  RepoTemplateOption,
  RepoVisibility,
  UserPublic,
} from "@octanest/api-client";
import { useState } from "octane";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { Button, buttonVariants } from "@/components/ui/button";
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
import { Textarea } from "@/components/ui/textarea";
import { apiClient } from "@/lib/api-client";
import { listSpdxLicenseOptions } from "@/lib/spdx-licenses";
import {
  fetchRepoCreateDefaults,
  fetchSessionMe,
} from "@/lib/ssr-auth";
import { cn } from "@/lib/utils";

export type NewLoaderData = {
  user: UserPublic;
  defaults: RepoCreateDefaults | null;
};

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";
const NAME_HELPER =
  "Letters, digits, hyphen, underscore, and period. Reserved names are rejected.";
const DUPLICATE_NAME_ERROR =
  "A repository with this name already exists. Choose a different name.";

function TemplateSelect({
  id,
  label,
  value,
  options,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (next: string) => void;
}) @{
  const current = options.find((o) => o.value === value) ?? options[0]!;

  <div className="flex flex-col gap-2">
    <Label htmlFor={id}>{label as string}</Label>
    <SelectRoot
      value={value}
      onValueChange={(next) => {
        if (typeof next === "string") onChange(next);
      }}
    >
      <SelectTrigger id={id} className="w-full" aria-label={label}>
        <SelectValue>{current.label as string}</SelectValue>
        <SelectIcon>
          <ChevronDown aria-hidden size={16} />
        </SelectIcon>
      </SelectTrigger>
      <SelectPortal>
        <SelectPositioner sideOffset={4}>
          <SelectPopup className="max-h-72 overflow-y-auto">
            <SelectList>
              @for (const opt of options; key opt.value) {
                <SelectItem value={opt.value}>
                  <SelectItemIndicator>
                    <Check aria-hidden size={16} className="text-primary" />
                  </SelectItemIndicator>
                  <SelectItemText>{opt.label as string}</SelectItemText>
                </SelectItem>
              }
            </SelectList>
          </SelectPopup>
        </SelectPositioner>
      </SelectPortal>
    </SelectRoot>
  </div>
}

function groupTemplateOptions(
  items: RepoTemplateOption[],
): { value: string; label: string }[] {
  const sorted = [...items].sort((a, b) => {
    const g = a.group.localeCompare(b.group);
    if (g !== 0) return g;
    return a.label.localeCompare(b.label);
  });
  return [
    { value: "none", label: "None" },
    ...sorted.map((item) => ({
      value: item.id,
      label: `${item.group} — ${item.label}`,
    })),
  ];
}

export const Route = createFileRoute("/new")({
  component: NewPage,
  head: () => ({ meta: [{ title: "New repository · Octanest" }] }),
  beforeLoad: async () => {
    try {
      const me = await fetchSessionMe();
      if (!me.ok) {
        throw redirect({ href: "/login?returnTo=/new" });
      }
    } catch (e) {
      if (e && typeof e === "object" && "href" in e) throw e;
      throw redirect({ href: "/login?returnTo=/new" });
    }
  },
  loader: async (): Promise<NewLoaderData> => {
    const me = await fetchSessionMe();
    if (!me.ok) {
      throw redirect({ href: "/login?returnTo=/new" });
    }
    let defaults: RepoCreateDefaults | null = null;
    if (me.data.email_verified) {
      const res = await fetchRepoCreateDefaults();
      if (res.ok) {
        defaults = res.data;
      }
    }
    return { user: me.data, defaults };
  },
});

/** Exported for /new Wave 0 integration tests (D-11 verify wall). */
export function NewPage() @{
  const data = useLoaderData({ from: "/new" }) as NewLoaderData | undefined;
  const user = data?.user;
  const showWall = !user || !user.email_verified;

  const stackOptions = groupTemplateOptions(data?.defaults?.stacks ?? []);
  const gitignoreOptions = groupTemplateOptions(data?.defaults?.gitignores ?? []);
  // Native <select> for full SPDX list — custom Select mounts every item and stalls.
  const licenseOptions = listSpdxLicenseOptions();

  const initialVisibility: RepoVisibility =
    data?.defaults?.default_visibility === "private" ? "private" : "public";

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [visibility, setVisibility] = useState<RepoVisibility>(initialVisibility);
  const [stackId, setStackId] = useState("none");
  const [licenseId, setLicenseId] = useState("none");
  const [gitignoreId, setGitignoreId] = useState("none");
  const [nameError, setNameError] = useState("");
  const [formError, setFormError] = useState("");
  const [pending, setPending] = useState(false);

  const ownerLabel = user?.username ? `@${user.username}` : "you";

  async function createRepo() {
    setNameError("");
    setFormError("");
    const trimmed = name.trim();
    if (!trimmed) {
      setNameError("Repository name is required.");
      return;
    }
    setPending(true);
    try {
      const res = await apiClient.repo.create({
        name: trimmed,
        description: description.trim() || null,
        visibility,
        stack_id: stackId === "none" ? null : stackId,
        license_id: licenseId === "none" ? null : licenseId,
        gitignore_id: gitignoreId === "none" ? null : gitignoreId,
      });
      if (!res.ok) {
        const code = res.error.code;
        if (code === "repo.name_taken") {
          setNameError(DUPLICATE_NAME_ERROR);
        } else if (code === "repo.invalid_name") {
          setNameError(res.error.message);
        } else if (code === "auth.email_unverified") {
          setFormError("Verify your email before creating a repository.");
        } else {
          setFormError(res.error.message || NETWORK_ERROR);
        }
        setPending(false);
        return;
      }
      window.location.assign(
        `/${res.data.owner_username}/${res.data.name}`,
      );
    } catch {
      setFormError(NETWORK_ERROR);
      setPending(false);
    }
  }

  @if (showWall) {
    <AuthShell
      title="Verify your email"
      support="Verify your email before creating a repository."
    >
      <a
        href="/verify"
        className={cn(buttonVariants({ variant: "default" }))}
      >
        Verify email
      </a>
    </AuthShell>
  } @else {
    <div className="mx-auto max-w-2xl px-4 py-16">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
        Create a new repository
      </h1>
      <p className="mt-2 text-[16px] text-muted-foreground">
        Owner is you for now. Organizations come in a later phase.
      </p>

      <form
        className="mt-8 flex flex-col gap-6"
        method="post"
        action="#"
        onSubmit={(e) => {
          e.preventDefault();
          void createRepo();
        }}
      >
        @if (formError) {
          <AuthErrorBanner message={formError} />
        }

        <div className="flex flex-col gap-2">
          <Label>Owner</Label>
          <p className="text-[16px] text-foreground">{ownerLabel as string}</p>
        </div>

        <div className="flex flex-col gap-2">
          <Label htmlFor="repo-name">Repository name</Label>
          <Input
            id="repo-name"
            name="name"
            autoComplete="off"
            value={name}
            aria-invalid={nameError ? true : undefined}
            onInput={(ev) => {
              setName((ev.currentTarget as HTMLInputElement).value);
              if (nameError) setNameError("");
            }}
            required
          />
          <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
            {NAME_HELPER}
          </p>
          @if (nameError) {
            <p role="alert" className="text-[14px] text-destructive">
              {nameError as string}
            </p>
          }
        </div>

        <div className="flex flex-col gap-2">
          <Label htmlFor="repo-description">Description (optional)</Label>
          <Textarea
            id="repo-description"
            name="description"
            rows={3}
            className="max-w-full break-words [overflow-wrap:anywhere]"
            value={description}
            onInput={(ev) =>
              setDescription((ev.currentTarget as HTMLTextAreaElement).value)
            }
          />
        </div>

        <fieldset className="flex flex-col gap-2">
          <legend className="text-[14px] font-normal leading-[1.4] text-foreground">
            Visibility
          </legend>
          <div
            className="flex flex-wrap gap-2"
            role="group"
            aria-label="Visibility"
          >
            <Button
              type="button"
              variant={visibility === "public" ? "default" : "secondary"}
              aria-pressed={visibility === "public"}
              onClick={() => setVisibility("public")}
            >
              Public
            </Button>
            <Button
              type="button"
              variant={visibility === "private" ? "default" : "secondary"}
              aria-pressed={visibility === "private"}
              onClick={() => setVisibility("private")}
            >
              Private
            </Button>
          </div>
        </fieldset>

        <div className="flex flex-col gap-4">
          <p className="text-[14px] font-normal text-muted-foreground">
            Templates (optional)
          </p>
          <TemplateSelect
            id="repo-stack"
            label="Stack template"
            value={stackId}
            options={stackOptions}
            onChange={setStackId}
          />
          <div className="flex flex-col gap-2">
            <Label htmlFor="repo-license">License</Label>
            <select
              id="repo-license"
              aria-label="License"
              className="h-11 w-full rounded-md border border-input bg-card px-3 text-[14px] font-normal text-foreground outline-none transition-colors duration-150 hover:border-foreground/25 focus-visible:ring-2 focus-visible:ring-ring"
              value={licenseId}
              onChange={(ev) =>
                setLicenseId((ev.currentTarget as HTMLSelectElement).value)
              }
            >
              @for (const opt of licenseOptions; key opt.id) {
                <option value={opt.id}>{opt.label as string}</option>
              }
            </select>
          </div>
          <TemplateSelect
            id="repo-gitignore"
            label=".gitignore"
            value={gitignoreId}
            options={gitignoreOptions}
            onChange={setGitignoreId}
          />
        </div>

        <Button
          type="button"
          disabled={pending}
          onClick={() => void createRepo()}
        >
          @if (pending) {
            <>Creating…</>
          } @else {
            <>Create repository</>
          }
        </Button>
      </form>
    </div>
  }
}
