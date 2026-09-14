import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

type Props = {
  confirmLabel: string;
  onCancel: () => void;
  onConfirm: (confirm: string) => void;
};

/** Type-to-confirm delete for package versions (D-PKG-12). */
export function DeleteVersionDialog(props: Props) @{
  const [value, setValue] = useState("");
  const ok = value === props.confirmLabel;

  @{
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      role="dialog"
      aria-modal="true"
      data-testid="delete-version-dialog"
    >
      <div className="w-full max-w-md rounded-lg border border-border bg-background p-6 shadow-lg">
        <h2 className="text-[18px] font-semibold text-foreground">Delete version</h2>
        <p className="mt-2 text-[14px] text-muted-foreground">
          Type <code className="text-foreground">{props.confirmLabel as string}</code> to confirm.
        </p>
        <div className="mt-4 space-y-2">
          <Label for="pkg-confirm">Confirmation</Label>
          <Input
            id="pkg-confirm"
            value={value as string}
            onInput={(e: any) => setValue(e.currentTarget.value)}
            autocomplete="off"
          />
        </div>
        <div className="mt-6 flex justify-end gap-2">
          <Button type="button" variant="secondary" onClick={props.onCancel}>
            Cancel
          </Button>
          <Button
            type="button"
            variant="destructive"
            disabled={!ok}
            onClick={() => props.onConfirm(value)}
          >
            Delete
          </Button>
        </div>
      </div>
    </div>
  }
}
