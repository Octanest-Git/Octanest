import type { CollaboratorPermission, RepoCollaboratorPublic } from "@oxidean/api-client";
import { Check, ChevronDown } from "@octanejs/lucide";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useForm } from "@octanejs/tanstack-form";
import { useState } from "octane";
import { AuthErrorBanner } from "@/components/auth-shell";
import { MemberLookup } from "@/components/org/member-lookup";
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

const NETWORK_ERROR = "Can't reach Oxidean. Check your connection and try again.";

const PERM_OPTIONS: { value: CollaboratorPermission; label: string }[] = [
  { value: "read", label: "Read" },
  { value: "write", label: "Write" },
  { value: "admin", label: "Admin" },
];

function permLabel(p: CollaboratorPermission): string {
  return PERM_OPTIONS.find((o) => o.value === p)?.label ?? p;
}

function collaboratorsQueryKey(owner: string, name: string) {
  return ["repo", "collaborators", owner, name] as const;
}

function PermissionSelect({
  id,
  value,
  onChange,
  disabled,
  ariaLabel,
}: {
  id: string;
  value: CollaboratorPermission;
  onChange: (p: CollaboratorPermission) => void;
  disabled?: boolean;
  ariaLabel?: string;
}) @{
  <SelectRoot
    value={value}
    onValueChange={(next) => {
      if (typeof next === "string") onChange(next as CollaboratorPermission);
    }}
  >
    <SelectTrigger
      id={id}
      className="min-w-[8rem]"
      aria-label={ariaLabel ?? "Permission"}
      disabled={disabled}
    >
      <SelectValue>{permLabel(value) as string}</SelectValue>
      <SelectIcon>
        <ChevronDown aria-hidden size={16} />
      </SelectIcon>
    </SelectTrigger>
    {/* octane-dom-race-ok: Base UI SelectPortal has no keepMounted escape
        hatch; popup is anchor-scoped under the Select root. */}
    <SelectPortal>
      <SelectPositioner sideOffset={4}>
        <SelectPopup>
          <SelectList>
            @for (const opt of PERM_OPTIONS; key opt.value) {
              <SelectItem value={opt.value}>
                <SelectItemIndicator>
                  <Check aria-hidden size={16} className="text-primary" />
                </SelectItemIndicator>
                <SelectItemText>{opt.label as string}</SelectItemText>
              </SelectItem>;
            }
          </SelectList>
        </SelectPopup>
      </SelectPositioner>
    </SelectPortal>
  </SelectRoot>;
}

export type CollaboratorsPanelProps = {
  owner: string;
  name: string;
};

/**
 * Repo Collaborators settings section (ORG-03 / D-ORG-02c / D-ORG-04).
 * Parent route must gate on repo.can_admin — this panel assumes admin access.
 */
export function CollaboratorsPanel({ owner, name }: CollaboratorsPanelProps) @{
  const queryClient = useQueryClient();
  const listQ = useQuery({
    queryKey: collaboratorsQueryKey(owner, name),
    queryFn: async () => {
      const res = await apiClient.repo.collaborators.list({ owner, name });
      if (!res.ok) {
        throw new Error(res.error.message || NETWORK_ERROR);
      }
      return res.data;
    },
  });

  const collaborators: RepoCollaboratorPublic[] = listQ.data?.collaborators ?? [];

  const [addError, setAddError] = useState("");
  const [addPending, setAddPending] = useState(false);

  const addForm = useForm({
    defaultValues: {
      username: "",
      permission: "write" as CollaboratorPermission,
    },
    onSubmit: async ({ value }) => {
      setAddError("");
      const username = value.username.trim();
      if (!username) {
        setAddError("Username is required.");
        return;
      }
      setAddPending(true);
      try {
        const res = await apiClient.repo.collaborators.add({
          owner,
          name,
          username,
          permission: value.permission,
        });
        if (!res.ok) {
          setAddError(res.error.message || NETWORK_ERROR);
          setAddPending(false);
          return;
        }
        addForm.reset();
        setShowAdd(false);
        setAddPending(false);
        await refresh();
      } catch {
        setAddError(NETWORK_ERROR);
        setAddPending(false);
      }
    },
  });
  const [showAdd, setShowAdd] = useState(false);
  const [rowError, setRowError] = useState("");
  const [removeTarget, setRemoveTarget] = useState<RepoCollaboratorPublic | null>(null);
  const [removePending, setRemovePending] = useState(false);
  const [removeError, setRemoveError] = useState("");

  async function refresh() {
    await queryClient.invalidateQueries({
      queryKey: collaboratorsQueryKey(owner, name),
    });
  }

  async function updatePermission(row: RepoCollaboratorPublic, permission: CollaboratorPermission) {
    if (permission === row.permission) return;
    setRowError("");
    try {
      const res = await apiClient.repo.collaborators.update({
        owner,
        name,
        user_id: row.user_id,
        permission,
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

  async function confirmRemove() {
    if (!removeTarget) return;
    setRemoveError("");
    setRemovePending(true);
    try {
      const res = await apiClient.repo.collaborators.remove({
        owner,
        name,
        user_id: removeTarget.user_id,
      });
      if (!res.ok) {
        setRemoveError(res.error.message || NETWORK_ERROR);
        setRemovePending(false);
        return;
      }
      setRemovePending(false);
      setRemoveTarget(null);
      await refresh();
    } catch {
      setRemoveError(NETWORK_ERROR);
      setRemovePending(false);
    }
  }

  <section className="mt-12 flex flex-col gap-4" aria-labelledby="collaborators-heading">
    <h2
      id="collaborators-heading"
      className="font-[family-name:var(--font-display)] text-[18px] font-semibold text-foreground"
    >
      Collaborators
    </h2>
    <p className="text-[14px] text-muted-foreground">
      Grant people direct access to this repository with read, write, or admin permission.
    </p>

    @if (rowError) {
      <AuthErrorBanner message={rowError} />;
    }

    @if (listQ.isPending) {
      <p className="text-[14px] text-muted-foreground">Loading collaborators…</p>;
    } @else if (listQ.isError) {
      <p role="alert" className="text-[14px] text-destructive">
        {(listQ.error instanceof Error ? listQ.error.message : NETWORK_ERROR) as string}
      </p>;
    } @else if (collaborators.length === 0 && !showAdd) {
      <>
        <p className="text-[14px] text-muted-foreground">
          No collaborators yet. Add someone by username to share access.
        </p>
        <Button type="button" className="h-11 w-fit" onClick={() => setShowAdd(true)}>
          Add collaborator
        </Button>
      </>;
    } @else {
      <>
        <div className="overflow-x-auto">
          <table className="w-full min-w-[28rem] border-collapse text-left text-[14px]">
            <thead>
              <tr className="border-b border-border text-muted-foreground">
                <th className="py-2 pr-4 font-medium">User</th>
                <th className="py-2 pr-4 font-medium">Permission</th>
                <th className="py-2 font-medium">
                  <span className="sr-only">Actions</span>
                </th>
              </tr>
            </thead>
            <tbody>
              @for (const c of collaborators; key c.user_id) {
                <tr className="border-b border-border">
                  <td className="py-3 pr-4 font-medium text-foreground">
                    {"@" + (c.username as string)}
                  </td>
                  <td className="py-3 pr-4">
                    <PermissionSelect
                      id={`collab-perm-${c.user_id}`}
                      value={c.permission}
                      onChange={(p) => void updatePermission(c, p)}
                    />
                  </td>
                  <td className="py-3">
                    <Button
                      type="button"
                      variant="ghost"
                      className="h-11 text-destructive"
                      onClick={() => {
                        setRemoveError("");
                        setRemoveTarget(c);
                      }}
                    >
                      Remove
                    </Button>
                  </td>
                </tr>;
              }
            </tbody>
          </table>
        </div>
        @if (!showAdd) {
          <Button type="button" className="h-11 w-fit" onClick={() => setShowAdd(true)}>
            Add collaborator
          </Button>;
        }
      </>;
    }

    @if (showAdd) {
      <form
        className="mt-2 flex flex-col gap-4 rounded-md border border-border p-4"
        method="post"
        action="#"
        onSubmit={(e) => {
          e.preventDefault();
          void addForm.handleSubmit();
        }}
      >
        <h3 className="text-[16px] font-semibold text-foreground">Add collaborator</h3>
        @if (addError) {
          <AuthErrorBanner message={addError} />;
        }
        <addForm.Field name="username">
          {(field) => (
            <MemberLookup
              id="collab-lookup"
              label="Username"
              value={field.state.value}
              onValueChange={field.handleChange}
            />
          )}
        </addForm.Field>
        <addForm.Subscribe selector={(s) => s.values.permission}>
          {(addPerm) => (
            <div className="flex flex-col gap-2">
              <Label htmlFor="collab-add-perm">Permission</Label>
              <PermissionSelect
                id="collab-add-perm"
                value={addPerm}
                onChange={(p) => addForm.setFieldValue("permission", p)}
                ariaLabel="Permission"
              />
            </div>
          )}
        </addForm.Subscribe>
        <div className="flex flex-wrap gap-2">
          <Button type="submit" className="h-11" disabled={addPending}>
            @if (addPending) {
              <>Adding…</>;
            } @else {
              <>Add collaborator</>;
            }
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-11"
            disabled={addPending}
            onClick={() => {
              setShowAdd(false);
              setAddError("");
              addForm.reset();
            }}
          >
            Cancel
          </Button>
        </div>
      </form>;
    }

    <AlertDialog
      open={!!removeTarget}
      onOpenChange={(open) => {
        if (removePending) return;
        if (!open) {
          setRemoveTarget(null);
          setRemoveError("");
        }
      }}
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Remove collaborator?</AlertDialogTitle>
          <AlertDialogDescription>
            {
              (removeTarget
                ? `Remove @${removeTarget.username} from this repository?`
                : "") as string
            }
          </AlertDialogDescription>
        </AlertDialogHeader>
        @if (removeError) {
          <p role="alert" className="text-[14px] text-destructive">
            {removeError as string}
          </p>;
        }
        <AlertDialogFooter>
          <AlertDialogCancel disabled={removePending}>Cancel</AlertDialogCancel>
          <Button
            type="button"
            variant="destructive"
            disabled={removePending}
            onClick={() => void confirmRemove()}
          >
            {(removePending ? "Removing…" : "Remove collaborator") as string}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </section>;
}
