import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  RadioGroup,
  RadioGroupItem,
} from "@/components/ui/radio-group";
import { apiClient } from "@/lib/api-client";

const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";

type PatClassicFormProps = {
  onCreated: (token: string) => void;
};

function tomorrowDateValue(): string {
  const d = new Date();
  d.setUTCDate(d.getUTCDate() + 1);
  return d.toISOString().slice(0, 10);
}

function expiryToIso(dateValue: string): string {
  // End of selected UTC day — server stores opaque timestamp string.
  return `${dateValue}T23:59:59Z`;
}

/** Classic PAT create form (D-05 / D-07 / D-16). */
export function PatClassicForm({ onCreated }: PatClassicFormProps) @{
  const [note, setNote] = useState("");
  const [repoChecked, setRepoChecked] = useState(true);
  const [expiryMode, setExpiryMode] = useState<"none" | "date">("none");
  const [expiryDate, setExpiryDate] = useState(tomorrowDateValue);
  const [noteError, setNoteError] = useState("");
  const [scopeError, setScopeError] = useState("");
  const [formError, setFormError] = useState("");
  const [pending, setPending] = useState(false);

  async function generate() {
    setNoteError("");
    setScopeError("");
    setFormError("");
    const trimmed = note.trim();
    if (!trimmed) {
      setNoteError("Note is required.");
      return;
    }
    if (!repoChecked) {
      setScopeError("Full control of private repositories is required.");
      return;
    }
    setPending(true);
    try {
      const res = await apiClient.pat.createClassic({
        name: trimmed,
        scopes: ["repo"],
        expires_at:
          expiryMode === "date" && expiryDate
            ? expiryToIso(expiryDate)
            : null,
      });
      if (!res.ok) {
        if (res.error.code === "auth.email_unverified") {
          setFormError(
            "Verify your email before creating a personal access token.",
          );
        } else if (res.error.code === "pat.note_required") {
          setNoteError("Note is required.");
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
  }

  <form
    method="post"
    action="#"
    className="mt-8 flex flex-col gap-6"
    onSubmit={(e) => {
      e.preventDefault();
    }}
  >
    <div className="flex flex-col gap-2">
      <Label htmlFor="pat-classic-note">Note</Label>
      <Input
        id="pat-classic-note"
        value={note as string}
        aria-label="Note"
        autoComplete="off"
        disabled={pending}
        onInput={(e) =>
          setNote((e.currentTarget as HTMLInputElement).value)
        }
      />
      <p className="text-[14px] text-muted-foreground">
        What’s this token for? e.g. laptop, CI
      </p>
      @if (noteError) {
        <p role="alert" className="text-[14px] text-destructive">
          {noteError as string}
        </p>
      }
    </div>

    <div className="flex flex-col gap-2">
      <p className="text-[14px] font-medium text-foreground">Scopes</p>
      <label className="flex cursor-pointer items-start gap-3">
        <Checkbox
          checked={repoChecked}
          disabled={pending}
          aria-label="Full control of private repositories"
          onCheckedChange={(checked) => setRepoChecked(checked === true)}
        />
        <span className="text-[14px] text-foreground">
          Full control of private repositories
        </span>
      </label>
      @if (scopeError) {
        <p role="alert" className="text-[14px] text-destructive">
          {scopeError as string}
        </p>
      }
    </div>

    <div className="flex flex-col gap-3">
      <p className="text-[14px] font-medium text-foreground">Expiration</p>
      <RadioGroup
        value={expiryMode}
        disabled={pending}
        onValueChange={(v) => {
          if (v === "none" || v === "date") setExpiryMode(v);
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
      @if (expiryMode === "date") {
        <Input
          type="date"
          value={expiryDate as string}
          min={tomorrowDateValue() as string}
          aria-label="Expiration date"
          disabled={pending}
          className="max-w-xs"
          onInput={(e) =>
            setExpiryDate((e.currentTarget as HTMLInputElement).value)
          }
        />
      }
    </div>

    @if (formError) {
      <p role="alert" className="text-[16px] text-destructive">
        {formError as string}
      </p>
    }

    <Button
      type="button"
      className="h-11 w-fit"
      disabled={pending || !repoChecked}
      onClick={() => void generate()}
    >
      {(pending ? "Generating…" : "Generate token") as string}
    </Button>
  </form>
}
