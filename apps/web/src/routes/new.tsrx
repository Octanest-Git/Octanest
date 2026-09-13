import {
  createFileRoute,
  redirect,
  useLoaderData,
} from "@octanejs/tanstack-router";
import type {
  RepoCreateDefaults,
  RepoVisibility,
  UserPublic,
} from "@octanest/api-client";
import { useState } from "octane";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { TemplatePicker } from "@/components/repo/template-picker";
import { Button, buttonVariants } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { apiClient } from "@/lib/api-client";
import { listLicensePickerOptions } from "@/lib/spdx-licenses";
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

  const stacks = data?.defaults?.stacks ?? [];
  const gitignores = data?.defaults?.gitignores ?? [];
  const licenseOptions = listLicensePickerOptions();

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
        // Send explicit "none" so stack default gitignore is not re-applied server-side.
        gitignore_id: gitignoreId,
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

        <div className="flex flex-col gap-5">
          <div>
            <p className="text-[14px] font-semibold text-foreground">
              Templates (optional)
            </p>
            <p className="mt-1 text-[13px] text-muted-foreground">
              Seed an initial commit with a stack, license, and ignore rules —
              or leave them as None for an empty Quick setup repo.
            </p>
          </div>
          <TemplatePicker
            id="repo-stack"
            label="Stack template"
            description="Project skeleton files committed on create. Picking a stack also selects a matching .gitignore (changeable below)."
            value={stackId}
            options={stacks}
            noneLabel="None"
            noneDescription="No stack files — empty repo with Quick setup."
            onChange={(id) => {
              setStackId(id);
              if (id === "none") {
                setGitignoreId("none");
                return;
              }
              const pack = stacks.find((s) => s.id === id);
              setGitignoreId(pack?.default_gitignore ?? "none");
            }}
          />
          <TemplatePicker
            id="repo-license"
            label="License"
            description="LICENSE file for the initial commit. Browse popular licenses or search the full SPDX catalog."
            value={licenseId}
            options={licenseOptions}
            noneLabel="None"
            noneDescription="No LICENSE in the initial commit."
            browseGroups={["Popular"]}
            searchPlaceholder="Search SPDX licenses…"
            onChange={setLicenseId}
          />
          <TemplatePicker
            id="repo-gitignore"
            label=".gitignore"
            description="Language and tooling ignore rules. Auto-filled from the stack; set to None to skip."
            value={gitignoreId}
            options={gitignores}
            noneLabel="None"
            noneDescription="No .gitignore in the initial commit."
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
