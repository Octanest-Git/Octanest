import {
  createFileRoute,
  redirect,
  useLoaderData,
} from "@octanejs/tanstack-router";
import type { RepoVisibility, UserPublic } from "@octanest/api-client";
import { useState } from "octane";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { Button, buttonVariants } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { apiClient } from "@/lib/api-client";
import { fetchSessionMe } from "@/lib/ssr-auth";
import { cn } from "@/lib/utils";

export type NewLoaderData = {
  user: UserPublic;
};

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";
const NAME_HELPER =
  "Letters, digits, hyphen, underscore, and period. Reserved names are rejected.";

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
    return { user: me.data };
  },
});

/** Exported for /new Wave 0 integration tests (D-11 verify wall). */
export function NewPage() @{
  const data = useLoaderData({ from: "/new" }) as NewLoaderData | undefined;
  const user = data?.user;
  const showWall = !user || !user.email_verified;

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [visibility, setVisibility] = useState<RepoVisibility>("public");
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
      });
      if (!res.ok) {
        const code = res.error.code;
        if (code === "repo.name_taken" || code === "repo.invalid_name") {
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

        <div className="flex flex-col gap-3 rounded-md border border-border bg-card/40 px-4 py-3">
          <p className="text-[14px] font-normal text-muted-foreground">
            Templates (placeholders — full pickers in a later plan)
          </p>
          <div className="grid gap-2 text-[14px] text-foreground sm:grid-cols-3">
            <p>
              <span className="text-muted-foreground">Stack: </span>None
            </p>
            <p>
              <span className="text-muted-foreground">License: </span>None
            </p>
            <p>
              <span className="text-muted-foreground">.gitignore: </span>None
            </p>
          </div>
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
