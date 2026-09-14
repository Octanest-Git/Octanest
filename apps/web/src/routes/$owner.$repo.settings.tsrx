import {
  createFileRoute,
  useLoaderData,
  useParams,
} from "@octanejs/tanstack-router";
import type { RepoVisibility } from "@octanest/api-client";
import { useState } from "octane";
import { CollaboratorsPanel } from "@/components/repo/collaborators-panel";
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
import { apiClient } from "@/lib/api-client";
import {
  type RepoLayoutLoaderData,
  useRepoStore,
} from "@/lib/repo-store";
import { toastSuccess } from "@/lib/toast";

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
  "Only people with access can see this private repository.";
const SOFT_DELETE_BODY =
  "Removes this repository from Octanest. Files may be purged from disk later. Type the repository name to confirm.";

/** Admin settings — visibility, collaborators, soft-delete (D-ORG-04 / ORG-03). */
export function RepoSettingsPage() @{
  const params = useParams({ from: "/$owner/$repo/settings" }) as {
    owner: string;
    repo: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const layout = useLoaderData({ from: "/$owner/$repo" }) as
    | RepoLayoutLoaderData
    | undefined;
  const setRepo = useRepoStore((s) => s.setRepo);

  const repo = layout?.repo ?? null;
  const layoutError = layout?.status === "error" ? layout.message : "";
  // Gate on API can_admin — not me.id === owner_id (D-ORG-04 / T-10-02).
  const canAdmin = !!repo?.can_admin;
  const showNotFound =
    !layout ||
    layout.status === "not_found" ||
    (layout.status === "ok" && !!repo && !canAdmin);

  const [visibility, setVisibility] = useState<RepoVisibility>(
    () => repo?.visibility ?? "public",
  );
  const [visPending, setVisPending] = useState(false);
  const [visError, setVisError] = useState("");
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [confirmName, setConfirmName] = useState("");
  const [deletePending, setDeletePending] = useState(false);
  const [deleteError, setDeleteError] = useState("");

  async function saveVisibility() {
    if (!repo) return;
    setVisError("");
    setVisPending(true);
    try {
      const res = await apiClient.repo.updateVisibility({
        owner,
        name: repoName,
        visibility,
      });
      if (!res.ok) {
        setVisError(res.error.message || "Could not update visibility.");
        setVisPending(false);
        return;
      }
      setVisibility(res.data.visibility);
      setRepo(res.data);
      toastSuccess("Visibility updated.");
      setVisPending(false);
    } catch {
      setVisError(NETWORK_ERROR);
      setVisPending(false);
    }
  }

  async function submitDelete() {
    if (!repo) return;
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

  @if (showNotFound) {
    <RepoNotFound />
  } @else if (layoutError) {
    <div className="max-w-lg py-6 text-center">
      <p role="alert" className="text-[16px] text-destructive">
        {layoutError as string}
      </p>
    </div>
  } @else if (repo && canAdmin) {
    <>
      <RepoChrome
        repo={repo}
        active="settings"
        showSettings={true}
      />

      <div className="max-w-3xl">
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
            disabled={visPending || visibility === repo.visibility}
            onClick={() => void saveVisibility()}
            className="w-full sm:w-auto"
          >
            {(visPending ? "Saving…" : "Save visibility") as string}
          </Button>
        </section>

        <CollaboratorsPanel owner={owner} name={repoName} />

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
      </div>

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
                deletePending || confirmName.trim() !== repo.name
              }
              onClick={() => void submitDelete()}
            >
              {(deletePending ? "Deleting…" : "Delete repository") as string}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  }
}
