import { useForm } from "@octanejs/tanstack-form";
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
  return `${dateValue}T23:59:59Z`;
}

/** Classic PAT create form (D-05 / D-07 / D-16). */
export function PatClassicForm({ onCreated }: PatClassicFormProps) @{
  const [noteError, setNoteError] = useState("");
  const [scopeError, setScopeError] = useState("");
  const [formError, setFormError] = useState("");
  const [pending, setPending] = useState(false);

  const form = useForm({
    defaultValues: {
      note: "",
      repoChecked: true,
      packageRead: false,
      packageWrite: false,
      expiryMode: "none" as "none" | "date",
      expiryDate: tomorrowDateValue(),
    },
    onSubmit: async ({ value }) => {
      setNoteError("");
      setScopeError("");
      setFormError("");
      const trimmed = value.note.trim();
      if (!trimmed) {
        setNoteError("Note is required.");
        return;
      }
      if (!value.repoChecked) {
        setScopeError("Full control of private repositories is required.");
        return;
      }
      setPending(true);
      try {
        const scopes: string[] = ["repo"];
        if (value.packageWrite) {
          scopes.push("package:write");
        } else if (value.packageRead) {
          scopes.push("package:read");
        }
        const res = await apiClient.pat.createClassic({
          name: trimmed,
          scopes: scopes as ("repo" | "package:read" | "package:write")[],
          expires_at:
            value.expiryMode === "date" && value.expiryDate
              ? expiryToIso(value.expiryDate)
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
    },
  });

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
          <Label htmlFor="pat-classic-note">Note</Label>
          <Input
            id="pat-classic-note"
            value={field.state.value}
            aria-label="Note"
            autoComplete="off"
            disabled={pending}
            onBlur={field.handleBlur}
            onInput={(e) =>
              field.handleChange((e.currentTarget as HTMLInputElement).value)
            }
          />
          <p className="text-[14px] text-muted-foreground">
            What's this token for? e.g. laptop, CI
          </p>
          @if (noteError) {
            <p role="alert" className="text-[14px] text-destructive">
              {noteError as string}
            </p>
          }
        </div>
      )}
    </form.Field>

    <form.Subscribe selector={(s) => s.values}>
      {(values) => (
        <div className="flex flex-col gap-2">
          <p className="text-[14px] font-medium text-foreground">Scopes</p>
          <label className="flex cursor-pointer items-start gap-3">
            <Checkbox
              checked={values.repoChecked}
              disabled={pending}
              aria-label="Full control of private repositories"
              onCheckedChange={(checked) =>
                form.setFieldValue("repoChecked", checked === true)
              }
            />
            <span className="text-[14px] text-foreground">
              Full control of private repositories
            </span>
          </label>
          <label className="flex cursor-pointer items-start gap-3">
            <Checkbox
              checked={values.packageRead || values.packageWrite}
              disabled={pending || values.packageWrite}
              aria-label="package:read"
              data-testid="scope-package-read"
              onCheckedChange={(checked) =>
                form.setFieldValue("packageRead", checked === true)
              }
            />
            <span className="text-[14px] text-foreground">
              package:read — download packages (does not include repo alone)
            </span>
          </label>
          <label className="flex cursor-pointer items-start gap-3">
            <Checkbox
              checked={values.packageWrite}
              disabled={pending}
              aria-label="package:write"
              data-testid="scope-package-write"
              onCheckedChange={(checked) => {
                const on = checked === true;
                form.setFieldValue("packageWrite", on);
                if (on) form.setFieldValue("packageRead", true);
              }}
            />
            <span className="text-[14px] text-foreground">
              package:write — publish and manage packages
            </span>
          </label>
          @if (scopeError) {
            <p role="alert" className="text-[14px] text-destructive">
              {scopeError as string}
            </p>
          }
        </div>
      )}
    </form.Subscribe>

    <form.Subscribe selector={(s) => s.values}>
      {(values) => (
        <div className="flex flex-col gap-3">
          <p className="text-[14px] font-medium text-foreground">Expiration</p>
          <RadioGroup
            value={values.expiryMode}
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
          @if (values.expiryMode === "date") {
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
                  onInput={(e) =>
                    field.handleChange((e.currentTarget as HTMLInputElement).value)
                  }
                />
              )}
            </form.Field>
          }
        </div>
      )}
    </form.Subscribe>

    @if (formError) {
      <p role="alert" className="text-[16px] text-destructive">
        {formError as string}
      </p>
    }

    <form.Subscribe selector={(s) => s.values.repoChecked}>
      {(repoChecked) => (
        <Button
          type="submit"
          className="h-11 w-fit"
          disabled={pending || !repoChecked}
        >
          {(pending ? "Generating…" : "Generate token") as string}
        </Button>
      )}
    </form.Subscribe>
  </form>
}
