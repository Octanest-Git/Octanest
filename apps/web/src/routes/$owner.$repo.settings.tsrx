import {
  createFileRoute,
  useParams,
} from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import type { RepoPublic, RepoVisibility } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { RepoChrome } from "@/components/repo/repo-chrome";
import { RepoNotFound } from "@/components/repo/repo-not-found";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import { toastSuccess } from "@/lib/toast";
import { authSessionQueryOptions } from "@/lib/session-queries";

export const Route = createFileRoute("/$owner/$repo/settings")({
  component: RepoSettingsPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Settings · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
});

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const PRIVATE_HELPER =
  "Only you can see this repository until organizations and collaborators ship.";
const SOFT_DELETE_BODY =
  "Removes this repository from Octanest. Files may be purged from disk later. Type the repository name to confirm.";

type Phase =
  | { kind: "loading" }
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | { kind: "ready"; repo: RepoPublic };

/** Owner-only visibility + soft-delete danger zone (D-26 / D-35). */
export function RepoSettingsPage() @{
  const params = useParams({ from: "/$owner/$repo/settings" }) as {
    owner: string;
    repo: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const me = useQuery(authSessionQueryOptions());
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });
  const [visibility, setVisibility] = useState<RepoVisibility>("public");
  const [visPending, setVisPending] = useState(false);
  const [visError, setVisError] = useState("");
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [confirmName, setConfirmName] = useState("");
  const [deletePending, setDeletePending] = useState(false);
  const [deleteError, setDeleteError] = useState("");

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setPhase({ kind: "loading" });
      try {
        const got = await apiClient.repo.get({ owner, name: repoName });
        if (cancelled) return;
        if (!got.ok) {
          setPhase(
            got.error.code === "repo.not_found"
              ? { kind: "not_found" }
              : {
                  kind: "error",
                  message: got.error.message || "Could not load repository.",
                },
          );
          return;
        }
        setPhase({ kind: "ready", repo: got.data });
        setVisibility(got.data.visibility);
      } catch {
        if (cancelled) return;
        setPhase({ kind: "error", message: NETWORK_ERROR });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [owner, repoName]);

  const isOwner =
    phase.kind === "ready" &&
    me.isSuccess &&
    me.data !== null &&
    me.data.id === phase.repo.owner_id;

  // Non-owner (incl. anonymous) → identical 404 once session resolves.
  const showNotFound =
    phase.kind === "not_found" ||
    (phase.kind === "ready" && me.isSuccess && !isOwner);

  async function saveVisibility() {
    if (phase.kind !== "ready") return;
    setVisError("");
    setVisPending(true);
    try {
      const res = await apiClient.repo.updateVisibility({
        owner,
        name: repoName,
        visibility,
      });
      if (!res.ok) {
        if (res.error.code === "repo.not_found") {
          setPhase({ kind: "not_found" });
          setVisPending(false);
          return;
        }
        setVisError(res.error.message || "Could not update visibility.");
        setVisPending(false);
        return;
      }
      setPhase({ kind: "ready", repo: res.data });
      setVisibility(res.data.visibility);
      toastSuccess("Visibility updated.");
      setVisPending(false);
    } catch {
      setVisError(NETWORK_ERROR);
      setVisPending(false);
    }
  }

  async function submitDelete() {
    if (phase.kind !== "ready") return;
    setDeleteError("");
    setDeletePending(true);
    try {
      const res = await apiClient.repo.softDelete({
        owner,
        name: repoName,
        confirmName: confirmName.trim(),
      });
      if (!res.ok) {
        setDeleteError(res.error.message || "Could not delete repository.");
        setDeletePending(false);
        return;
      }
      window.location.assign("/");
    } catch {
      setDeleteError(NETWORK_ERROR);
      setDeletePending(false);
    }
  }

  @if (phase.kind === "loading" || (phase.kind === "ready" && me.isPending)) {
    <div className="mx-auto max-w-3xl px-4 py-10">
      <Skeleton className="h-8 w-64" />
      <Skeleton className="mt-6 h-40 w-full" />
    </div>
  } @else if (showNotFound) {
    <RepoNotFound />
  } @else if (phase.kind === "error") {
    <div className="mx-auto max-w-lg px-4 py-16 text-center">
      <p role="alert" className="text-[16px] text-destructive">
        {phase.message as string}
      </p>
    </div>
  } @else if (phase.kind === "ready" && isOwner) {
    <div className="mx-auto max-w-3xl px-4 py-10">
      <RepoChrome
        repo={phase.repo}
        active="settings"
        showSettings={true}
      />

      <section className="mt-8 flex flex-col gap-4" aria-labelledby="vis-heading">
        <h2
          id="vis-heading"
          className="font-[family-name:var(--font-display)] text-[18px] font-semibold text-foreground"
        >
          Visibility
        </h2>
        <p className="text-[14px] text-muted-foreground">
          Choose who can see this repository.
        </p>
        <div className="flex flex-wrap gap-2" role="group" aria-label="Visibility">
          <Button
            type="button"
            variant={visibility === "public" ? "default" : "secondary"}
            aria-pressed={visibility === "public"}
            disabled={visPending}
            onClick={() => setVisibility("public")}
          >
            Public
          </Button>
          <Button
            type="button"
            variant={visibility === "private" ? "default" : "secondary"}
            aria-pressed={visibility === "private"}
            disabled={visPending}
            onClick={() => setVisibility("private")}
          >
            Private
          </Button>
        </div>
        @if (visibility === "private") {
          <p className="text-[14px] text-muted-foreground">{PRIVATE_HELPER as string}</p>
        }
        @if (visError) {
          <p role="alert" className="text-[14px] text-destructive">
            {visError as string}
          </p>
        }
        <Button
          type="button"
          disabled={visPending || visibility === phase.repo.visibility}
          onClick={() => void saveVisibility()}
          className="w-full sm:w-auto"
        >
          {(visPending ? "Saving…" : "Save visibility") as string}
        </Button>
      </section>

      <section
        className="mt-12 flex flex-col gap-4 rounded-md border border-destructive/40 p-4"
        aria-labelledby="danger-heading"
      >
        <h2
          id="danger-heading"
          className="font-[family-name:var(--font-display)] text-[18px] font-semibold text-destructive"
        >
          Danger zone
        </h2>
        <p className="text-[14px] leading-[1.4] text-muted-foreground">
          Soft-delete removes this repository from Octanest. Disk cleanup may run
          later.
        </p>
        <Button
          type="button"
          variant="destructive"
          className="w-full sm:w-auto"
          onClick={() => {
            setDeleteError("");
            setConfirmName("");
            setDeleteOpen(true);
          }}
        >
          Delete repository
        </Button>
      </section>

      <AlertDialog
        open={deleteOpen}
        onOpenChange={(open) => {
          if (deletePending) return;
          setDeleteOpen(open);
          if (!open) {
            setConfirmName("");
            setDeleteError("");
          }
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete repository?</AlertDialogTitle>
            <AlertDialogDescription>
              {SOFT_DELETE_BODY as string}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="flex flex-col gap-2">
            <Label htmlFor="soft-delete-confirm">Repository name</Label>
            <Input
              id="soft-delete-confirm"
              value={confirmName}
              onInput={(e) =>
                setConfirmName((e.currentTarget as HTMLInputElement).value)
              }
              autoComplete="off"
              disabled={deletePending}
              placeholder={repoName as string}
            />
          </div>
          @if (deleteError) {
            <p role="alert" className="text-[14px] text-destructive">
              {deleteError as string}
            </p>
          }
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deletePending}>Cancel</AlertDialogCancel>
            <Button
              type="button"
              variant="destructive"
              disabled={
                deletePending || confirmName.trim() !== phase.repo.name
              }
              onClick={() => void submitDelete()}
            >
              {(deletePending ? "Deleting…" : "Delete repository") as string}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  }
}
