import { useEffect, useState } from "octane";
import { useForm } from "@octanejs/tanstack-form";
import type { RepoPublic } from "@octanest/api-client";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

type PatFgFormProps = {
  onCreated: (token: string) => void;
};

function tomorrowDateValue(): string {
  const d = new Date();
  d.setUTCDate(d.getUTCDate() + 1);
  return d.toISOString().slice(0, 10);
}

function expiryToIso(dateValue: string): string {
  return `${dateValue}T23:59:59Z`;
}

function repoFullName(repo: RepoPublic): string {
  return `${repo.owner_username}/${repo.name}`;
}

/** Fine-grained PAT create form (D-05 / D-06 / D-07 / D-16 / T-08-06). */
export function PatFgForm({ onCreated }: PatFgFormProps) @{
  const [noteError, setNoteError] = useState("");
  const [repoError, setRepoError] = useState("");
  const [formError, setFormError] = useState("");
  const [pending, setPending] = useState(false);
  const [repos, setRepos] = useState<RepoPublic[] | null>(null);
  const [reposError, setReposError] = useState("");

  const form = useForm({
    defaultValues: {
      note: "",
      repoAccess: "selected" as "all" | "selected",
      selectedIds: [] as string[],
      contents: "read" as "read" | "write",
      packagesPerm: "none" as "none" | "read" | "write",
      expiryMode: "none" as "none" | "date",
      expiryDate: tomorrowDateValue(),
    },
    onSubmit: async ({ value }) => {
      setNoteError("");
      setRepoError("");
      setFormError("");
      const trimmed = value.note.trim();
      if (!trimmed) {
        setNoteError("Note is required.");
        return;
      }
      if (value.repoAccess === "selected" && value.selectedIds.length === 0) {
        setRepoError("Select at least one repository.");
        return;
      }
      setPending(true);
      try {
        const res = await apiClient.pat.createFineGrained({
          name: trimmed,
          repo_access: value.repoAccess,
          repository_ids: value.repoAccess === "selected" ? value.selectedIds : undefined,
          contents: value.contents,
          packages: value.packagesPerm === "none" ? null : value.packagesPerm,
          expires_at:
            value.expiryMode === "date" && value.expiryDate ? expiryToIso(value.expiryDate) : null,
        });
        if (!res.ok) {
          if (res.error.code === "auth.email_unverified") {
            setFormError("Verify your email before creating a personal access token.");
          } else if (res.error.code === "pat.note_required") {
            setNoteError("Note is required.");
          } else if (res.error.code === "pat.repos_required") {
            setRepoError("Select at least one repository.");
          } else {
            setFormError(res.error.message || NETWORK_ERROR);
          }
          setPending(false);
          return;
        }
        onCreated(res.data.token);
      } catch {
        setFormError(NETWORK_ERROR);
        setPending(false);
      }
    },
  });

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const res = await apiClient.repo.listMine();
        if (cancelled) return;
        if (!res.ok) {
          setReposError(res.error.message || NETWORK_ERROR);
          setRepos([]);
          return;
        }
        setRepos(res.data.repos);
      } catch {
        if (cancelled) return;
        setReposError(NETWORK_ERROR);
        setRepos([]);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  function toggleRepo(id: string, checked: boolean) {
    const current = form.getFieldValue("selectedIds");
    if (checked) {
      if (current.includes(id)) return;
      form.setFieldValue("selectedIds", [...current, id]);
    } else {
      form.setFieldValue(
        "selectedIds",
        current.filter((x) => x !== id),
      );
    }
  }

  const reposLoading = repos === null;
  const ownedRepos = repos ?? [];

  <form
    method="post"
    action="#"
    className="mt-8 flex flex-col gap-6"
    onSubmit={(e) => {
      e.preventDefault();
      void form.handleSubmit();
    }}
  >
    <form.Field name="note">
      {(field) => (
        <div className="flex flex-col gap-2">
          <Label htmlFor="pat-fg-note">Note</Label>
          <Input
            id="pat-fg-note"
            value={field.state.value}
            aria-label="Note"
            autoComplete="off"
            disabled={pending}
            onBlur={field.handleBlur}
            onInput={(e) => field.handleChange((e.currentTarget as HTMLInputElement).value)}
          />
          <p className="text-[14px] text-muted-foreground">
            What’s this token for? For example, laptop or CI.
          </p>
          @if (noteError) {
            <p role="alert" className="text-[14px] text-destructive">
              {noteError as string}
            </p>;
          }
        </div>
      )}
    </form.Field>

    <div className="flex flex-col gap-3">
      <p className="text-[14px] font-medium text-foreground">Repository access</p>
      <RadioGroup
        value={form.state.values.repoAccess}
        disabled={pending}
        onValueChange={(v) => {
          if (v === "all" || v === "selected") form.setFieldValue("repoAccess", v);
        }}
        className="gap-3"
      >
        <label className="flex cursor-pointer items-start gap-3">
          <RadioGroupItem value="all" className="mt-0.5" />
          <span className="flex flex-col gap-1">
            <span className="text-[14px] text-foreground">All repositories</span>
            <span className="text-[14px] text-muted-foreground">
              Includes all current and future repositories you own.
            </span>
          </span>
        </label>
        <label className="flex cursor-pointer items-start gap-3">
          <RadioGroupItem value="selected" className="mt-0.5" />
          <span className="text-[14px] text-foreground">Only select repositories</span>
        </label>
      </RadioGroup>

      @if (form.state.values.repoAccess === "selected") {
        <div className="mt-1 flex max-h-64 flex-col gap-2 overflow-y-auto rounded-md border border-border bg-card p-3">
          @if (reposLoading) {
            <>
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-3/4" />
            </>;
          } @else {
            @if (reposError) {
              <p role="alert" className="text-[14px] text-destructive">
                {reposError as string}
              </p>;
            } @else {
              @if (ownedRepos.length === 0) {
                <>
                  <p className="text-[16px] text-muted-foreground">
                    You don’t have any repositories yet.
                  </p>
                  <a
                    href="/new"
                    className="text-[14px] font-semibold text-primary no-underline hover:underline"
                  >
                    New repository
                  </a>
                </>;
              } @else {
                @for (const repo of ownedRepos; key repo.id) {
                  <label className="flex min-w-0 cursor-pointer items-center gap-3">
                    <Checkbox
                      checked={form.state.values.selectedIds.includes(repo.id)}
                      disabled={pending}
                      aria-label={repoFullName(repo)}
                      onCheckedChange={(checked) =>
                        toggleRepo(repo.id, checked === true)
                      }
                    />
                    <span
                      className="min-w-0 flex-1 truncate text-[14px] text-foreground"
                      title={repoFullName(repo) as string}
                    >
                      {repoFullName(repo) as string}
                    </span>
                    <Badge variant="outline">
                      @if (
                        repo.visibility === "private"
                      ) {
                        <>Private</>;
                      } @else {
                        <>Public</>;
                      }
                    </Badge>
                  </label>;
                }
              }
            }
          }
        </div>;
      }

      @if (repoError) {
        <p role="alert" className="text-[14px] text-destructive">
          {repoError as string}
        </p>;
      }
    </div>

    <div className="flex flex-col gap-3">
      <p className="text-[14px] font-medium text-foreground">Contents permission</p>
      <RadioGroup
        value={form.state.values.contents}
        disabled={pending}
        onValueChange={(v) => {
          if (v === "read" || v === "write") form.setFieldValue("contents", v);
        }}
        className="gap-3"
      >
        <label className="flex cursor-pointer items-center gap-3">
          <RadioGroupItem value="read" />
          <span className="text-[14px] text-foreground">Read-only</span>
        </label>
        <label className="flex cursor-pointer items-center gap-3">
          <RadioGroupItem value="write" />
          <span className="text-[14px] text-foreground">Read and write</span>
        </label>
      </RadioGroup>
      <p className="text-[14px] text-muted-foreground">
        Read-only allows clone and fetch. Read and write also allows push.
      </p>
    </div>

    <div className="flex flex-col gap-3" data-testid="fg-packages-perm">
      <p className="text-[14px] font-medium text-foreground">Packages permission</p>
      <RadioGroup
        value={form.state.values.packagesPerm}
        disabled={pending}
        onValueChange={(v) => {
          if (v === "none" || v === "read" || v === "write") {
            form.setFieldValue("packagesPerm", v);
          }
        }}
        className="gap-3"
      >
        <label className="flex cursor-pointer items-center gap-3">
          <RadioGroupItem value="none" />
          <span className="text-[14px] text-foreground">No package access</span>
        </label>
        <label className="flex cursor-pointer items-center gap-3">
          <RadioGroupItem value="read" />
          <span className="text-[14px] text-foreground">Packages read</span>
        </label>
        <label className="flex cursor-pointer items-center gap-3">
          <RadioGroupItem value="write" />
          <span className="text-[14px] text-foreground">Packages write</span>
        </label>
      </RadioGroup>
      <p className="text-[14px] text-muted-foreground">
        Repo scope alone does not grant package registry access.
      </p>
    </div>

    <div className="flex flex-col gap-3">
      <p className="text-[14px] font-medium text-foreground">Expiration</p>
      <RadioGroup
        value={form.state.values.expiryMode}
        disabled={pending}
        onValueChange={(v) => {
          if (v === "none" || v === "date") form.setFieldValue("expiryMode", v);
        }}
        className="gap-3"
      >
        <label className="flex cursor-pointer items-center gap-3">
          <RadioGroupItem value="none" />
          <span className="text-[14px] text-foreground">No expiration</span>
        </label>
        <label className="flex cursor-pointer items-center gap-3">
          <RadioGroupItem value="date" />
          <span className="text-[14px] text-foreground">Expires on</span>
        </label>
      </RadioGroup>
      @if (form.state.values.expiryMode === "date") {
        <form.Field name="expiryDate">
          {(field) => (
            <Input
              type="date"
              value={field.state.value}
              min={tomorrowDateValue() as string}
              aria-label="Expiration date"
              disabled={pending}
              className="max-w-xs"
              onBlur={field.handleBlur}
              onInput={(e) => field.handleChange((e.currentTarget as HTMLInputElement).value)}
            />
          )}
        </form.Field>;
      }
    </div>

    @if (formError) {
      <p role="alert" className="text-[16px] text-destructive">
        {formError as string}
      </p>;
    }

    <Button type="submit" className="h-11 w-fit" disabled={pending}>
      {(pending ? "Generating…" : "Generate token") as string}
    </Button>
  </form>;
}
