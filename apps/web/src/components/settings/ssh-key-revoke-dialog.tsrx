import type { SshKeyListItem } from "@octanest/api-client";
import { useState } from "octane";
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
import { apiClient } from "@/lib/api-client";

const REVOKE_ERROR =
  "Couldn't revoke SSH key. Check your connection and try again.";

type SshKeyRevokeDialogProps = {
  item: SshKeyListItem | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRevoked: () => void;
};

/** D-SSH-05 revoke confirm — Keep key / Revoke key (no type-to-confirm). */
export function SshKeyRevokeDialog({
  item,
  open,
  onOpenChange,
  onRevoked,
}: SshKeyRevokeDialogProps) @{
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const note = item?.title ?? "this key";

  async function confirmRevoke() {
    if (!item || pending) return;
    setError("");
    setPending(true);
    try {
      const res = await apiClient.sshKey.revoke({ id: item.id });
      if (!res.ok) {
        setError(REVOKE_ERROR);
        setPending(false);
        return;
      }
      setPending(false);
      onRevoked();
    } catch {
      setError(REVOKE_ERROR);
      setPending(false);
    }
  }

  <AlertDialog
    open={open}
    onOpenChange={(next) => {
      if (pending) return;
      if (!next) setError("");
      onOpenChange(next);
    }}
  >
    <AlertDialogContent className="max-h-[min(90vh,32rem)] overflow-y-auto">
      <AlertDialogHeader>
        <AlertDialogTitle>Revoke SSH key?</AlertDialogTitle>
        <AlertDialogDescription className="break-words whitespace-normal">
          {(`Revokes “${note}”. Git clients using this key will stop working. This can’t be undone.`) as string}
        </AlertDialogDescription>
      </AlertDialogHeader>
      @if (error) {
        <p role="alert" className="text-[14px] text-destructive">
          {error as string}
        </p>
      }
      <AlertDialogFooter>
        <AlertDialogCancel disabled={pending}>Keep key</AlertDialogCancel>
        <Button
          type="button"
          variant="destructive"
          disabled={pending}
          onClick={() => void confirmRevoke()}
        >
          {(pending ? "Revoking…" : "Revoke key") as string}
        </Button>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
}
