import {
  createFileRoute,
  useParams,
} from "@octanejs/tanstack-router";
import { useQuery } from "@octanejs/tanstack-query";
import type { RepoPublic, RepoRefEntry } from "@octanest/api-client";
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
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { apiClient } from "@/lib/api-client";
import {
  isBranchRef,
  shortOid,
  shortRefName,
  treeHref,
} from "@/lib/repo-browse";
import { authSessionQueryOptions } from "@/lib/session-queries";

export const Route = createFileRoute("/$owner/$repo/branches")({
  component: RepoBranchesPage,
  head: ({ params }) => ({
    meta: [
      {
        title: `Branches · ${(params as { owner: string }).owner}/${(params as { repo: string }).repo} · Octanest`,
      },
    ],
  }),
});

const DEFAULT_SOFT_PROTECT_TITLE =
  "Default branch can’t be renamed or deleted here.";

type Phase =
  | { kind: "loading" }
  | { kind: "not_found" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      repo: RepoPublic;
      branches: RepoRefEntry[];
    };

type DialogState =
  | { kind: "none" }
  | { kind: "create" }
  | { kind: "rename"; from: string }
  | { kind: "delete"; branch: string };

export function RepoBranchesPage() @{
  const params = useParams({ from: "/$owner/$repo/branches" }) as {
    owner: string;
    repo: string;
  };
  const owner = params.owner ?? "";
  const repoName = params.repo ?? "";
  const me = useQuery(authSessionQueryOptions());
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });
  const [branchName, setBranchName] = useState("");
  const [startRef, setStartRef] = useState("");
  const [confirmName, setConfirmName] = useState("");
  const [pending, setPending] = useState(false);
  const [formError, setFormError] = useState("");
  const [reloadKey, setReloadKey] = useState(0);

  const isOwner =
    phase.kind === "ready" &&
    me.isSuccess &&
    me.data !== null &&
    me.data.id === phase.repo.owner_id;

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
        const refsRes = await apiClient.repo.refs({
          owner,
          name: repoName,
        });
        if (cancelled) return;
        if (!refsRes.ok) {
          setPhase(
            refsRes.error.code === "repo.not_found"
              ? { kind: "not_found" }
              : {
                  kind: "error",
                  message: refsRes.error.message || "Could not load branches.",
                },
          );
          return;
        }
        const branches = refsRes.data.refs
          .filter((r) => isBranchRef(r.name))
          .sort((a, b) =>
            shortRefName(a.name).localeCompare(shortRefName(b.name), undefined, {
              sensitivity: "base",
            }),
          );
        setPhase({ kind: "ready", repo: got.data, branches });
      } catch {
        if (cancelled) return;
        setPhase({
          kind: "error",
          message: "Can't reach Octanest. Check your connection and try again.",
        });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [owner, repoName, reloadKey]);

  function openCreate() {
    if (phase.kind !== "ready") return;
    setFormError("");
    setBranchName("");
    setStartRef(phase.repo.default_branch || "main");
    setDialog({ kind: "create" });
  }

  function openRename(from: string) {
    setFormError("");
    setBranchName(from);
    setDialog({ kind: "rename", from });
  }

  function openDelete(branch: string) {
    setFormError("");
    setConfirmName("");
    setDialog({ kind: "delete", branch });
  }

  function closeDialog() {
    if (pending) return;
    setDialog({ kind: "none" });
    setFormError("");
    setConfirmName("");
  }

  async function submitCreate() {
    if (phase.kind !== "ready") return;
    const name = branchName.trim();
    if (!name) {
      setFormError("Enter a branch name.");
      return;
    }
    setPending(true);
    setFormError("");
    try {
      const res = await apiClient.repo.branchCreate({
        owner,
        name: repoName,
        branch: name,
        start: startRef.trim() || phase.repo.default_branch,
      });
      if (!res.ok) {
        setFormError(res.error.message || "Could not create branch.");
        setPending(false);
        return;
      }
      setPending(false);
      setDialog({ kind: "none" });
      setReloadKey((k) => k + 1);
    } catch {
      setFormError("Can't reach Octanest. Check your connection and try again.");
      setPending(false);
    }
  }

  async function submitRename() {
    if (phase.kind !== "ready" || dialog.kind !== "rename") return;
    const to = branchName.trim();
    if (!to) {
      setFormError("Enter a new branch name.");
      return;
    }
    setPending(true);
    setFormError("");
    try {
      const res = await apiClient.repo.branchRename({
        owner,
        name: repoName,
        from: dialog.from,
        to,
      });
      if (!res.ok) {
        setFormError(res.error.message || "Could not rename branch.");
        setPending(false);
        return;
      }
      setPending(false);
      setDialog({ kind: "none" });
      setReloadKey((k) => k + 1);
    } catch {
      setFormError("Can't reach Octanest. Check your connection and try again.");
      setPending(false);
    }
  }

  async function submitDelete() {
    if (phase.kind !== "ready" || dialog.kind !== "delete") return;
    if (confirmName.trim() !== dialog.branch) return;
    setPending(true);
    setFormError("");
    try {
      const res = await apiClient.repo.branchDelete({
        owner,
        name: repoName,
        branch: dialog.branch,
      });
      if (!res.ok) {
        setFormError(res.error.message || "Could not delete branch.");
        setPending(false);
        return;
      }
      setPending(false);
      setDialog({ kind: "none" });
      setConfirmName("");
      setReloadKey((k) => k + 1);
    } catch {
      setFormError("Can't reach Octanest. Check your connection and try again.");
      setPending(false);
    }
  }

  @if (phase.kind === "not_found") {
    <RepoNotFound />
  } @else {
    <div className="mx-auto max-w-6xl px-4 py-10">
      @if (phase.kind === "loading") {
        <div className="flex flex-col gap-4" aria-busy="true">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-12 w-full" />
          <Skeleton className="h-12 w-full" />
          <Skeleton className="h-12 w-full" />
        </div>
      } @else {
        @if (phase.kind === "error") {
          <div className="flex flex-col gap-4">
            <p role="alert" className="text-[16px] text-destructive">
              {phase.message as string}
            </p>
            <Button
              type="button"
              variant="secondary"
              onClick={() => setReloadKey((k) => k + 1)}
            >
              Try again
            </Button>
          </div>
        } @else {
          <>
            <RepoChrome repo={phase.repo} active="branches" />
            <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
              <h2 className="text-[18px] font-semibold text-foreground">
                Branches
              </h2>
              @if (isOwner) {
                <Button type="button" onClick={() => openCreate()}>
                  New branch
                </Button>
              }
            </div>

            @if (phase.branches.length === 0) {
              <p className="text-[16px] text-muted-foreground">
                No branches yet.
              </p>
            } @else {
              <ul className="divide-y divide-border border-y border-border">
                @for (const b of phase.branches; key b.name) {
                  <BranchRow
                    owner={owner}
                    repo={repoName}
                    branch={b}
                    defaultBranch={phase.repo.default_branch}
                    isOwner={isOwner}
                    onRename={(name) => openRename(name)}
                    onDelete={(name) => openDelete(name)}
                  />
                }
              </ul>
            }

            <Dialog
              open={dialog.kind === "create" || dialog.kind === "rename"}
              onOpenChange={(open) => {
                if (!open) closeDialog();
              }}
            >
              <DialogContent showCloseButton={!pending}>
                <DialogHeader>
                  <DialogTitle>
                    {(dialog.kind === "rename"
                      ? "Rename branch"
                      : "New branch") as string}
                  </DialogTitle>
                  <DialogDescription>
                    {(dialog.kind === "rename"
                      ? `Rename “${dialog.from}” to a new name.`
                      : "Create a branch from an existing ref.") as string}
                  </DialogDescription>
                </DialogHeader>
                <div className="flex flex-col gap-3">
                  <div className="flex flex-col gap-2">
                    <Label htmlFor="branch-name-input">Branch name</Label>
                    <Input
                      id="branch-name-input"
                      value={branchName}
                      onInput={(e) =>
                        setBranchName(
                          (e.currentTarget as HTMLInputElement).value,
                        )
                      }
                      autoComplete="off"
                      disabled={pending}
                    />
                  </div>
                  @if (dialog.kind === "create") {
                    <div className="flex flex-col gap-2">
                      <Label htmlFor="branch-start-input">Start point</Label>
                      <Input
                        id="branch-start-input"
                        value={startRef}
                        onInput={(e) =>
                          setStartRef(
                            (e.currentTarget as HTMLInputElement).value,
                          )
                        }
                        autoComplete="off"
                        disabled={pending}
                      />
                    </div>
                  }
                  @if (formError) {
                    <p role="alert" className="text-[14px] text-destructive">
                      {formError as string}
                    </p>
                  }
                </div>
                <DialogFooter>
                  <DialogClose
                    disabled={pending}
                    className="inline-flex h-11 items-center justify-center rounded-md border-2 border-secondary bg-secondary/5 px-4 text-[14px] font-semibold text-secondary"
                  >
                    Cancel
                  </DialogClose>
                  <Button
                    type="button"
                    disabled={pending}
                    onClick={() =>
                      void (dialog.kind === "rename"
                        ? submitRename()
                        : submitCreate())
                    }
                  >
                    {(pending
                      ? "Working…"
                      : dialog.kind === "rename"
                        ? "Rename branch"
                        : "Create branch") as string}
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>

            <AlertDialog
              open={dialog.kind === "delete"}
              onOpenChange={(open) => {
                if (!open) closeDialog();
              }}
            >
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>Delete branch?</AlertDialogTitle>
                  <AlertDialogDescription>
                    This deletes the branch from the repository. Type the branch
                    name to confirm.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="delete-branch-confirm">
                    {(dialog.kind === "delete"
                      ? `Type ${dialog.branch} to confirm`
                      : "Confirm") as string}
                  </Label>
                  <Input
                    id="delete-branch-confirm"
                    value={confirmName}
                    onInput={(e) =>
                      setConfirmName(
                        (e.currentTarget as HTMLInputElement).value,
                      )
                    }
                    autoComplete="off"
                    disabled={pending}
                  />
                  @if (formError) {
                    <p role="alert" className="text-[14px] text-destructive">
                      {formError as string}
                    </p>
                  }
                </div>
                <AlertDialogFooter>
                  <AlertDialogCancel disabled={pending}>Cancel</AlertDialogCancel>
                  <Button
                    type="button"
                    variant="destructive"
                    disabled={
                      pending ||
                      dialog.kind !== "delete" ||
                      confirmName.trim() !== dialog.branch
                    }
                    onClick={() => void submitDelete()}
                  >
                    {(pending ? "Deleting…" : "Delete branch") as string}
                  </Button>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </>
        }
      }
    </div>
  }
}

function BranchRow({
  owner,
  repo,
  branch,
  defaultBranch,
  isOwner,
  onRename,
  onDelete,
}: {
  owner: string;
  repo: string;
  branch: RepoRefEntry;
  defaultBranch: string;
  isOwner: boolean;
  onRename: (name: string) => void;
  onDelete: (name: string) => void;
}) @{
  const name = shortRefName(branch.name);
  const isDefault = name === defaultBranch;
  const tip = shortOid(branch.oid);
  const href = treeHref(owner, repo, name, "");

  <li className="flex flex-wrap items-center gap-x-4 gap-y-2 py-3">
    <div className="min-w-0 flex-1">
      <a
        href={href as string}
        className="block truncate text-[16px] font-semibold text-primary no-underline hover:underline"
        title={name as string}
      >
        {name as string}
      </a>
      <p className="mt-0.5 font-mono text-[13px] text-muted-foreground" title={branch.oid as string}>
        {tip as string}
        @if (isDefault) {
          <span className="ml-2 font-sans text-[13px] text-muted-foreground">
            · default
          </span>
        }
      </p>
    </div>
    @if (isOwner) {
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="secondary"
          disabled={isDefault}
          title={isDefault ? DEFAULT_SOFT_PROTECT_TITLE : undefined}
          onClick={() => {
            if (!isDefault) onRename(name);
          }}
        >
          Rename
        </Button>
        <Button
          type="button"
          variant="destructive"
          disabled={isDefault}
          title={isDefault ? DEFAULT_SOFT_PROTECT_TITLE : undefined}
          onClick={() => {
            if (!isDefault) onDelete(name);
          }}
        >
          Delete
        </Button>
      </div>
    }
  </li>
}
