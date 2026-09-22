import { gpgKeyAddMutationOptions } from "@octanest/api-client";
import { useMutation, useQueryClient } from "@octanejs/tanstack-query";
import { useForm } from "@octanejs/tanstack-form";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { apiClient } from "@/lib/api-client";

type GpgKeyAddFormProps = {
  disabled?: boolean;
  onAdded?: () => void;
};

/** Title + armored public key form. */
export function GpgKeyAddForm({ disabled = false, onAdded }: GpgKeyAddFormProps) @{
  const queryClient = useQueryClient();
  const [error, setError] = useState("");

  const form = useForm({
    defaultValues: {
      title: "",
      armored_public_key: "",
    },
    onSubmit: ({ value }) => {
      if (disabled || add.isPending) return;
      setError("");
      const t = value.title.trim();
      const k = value.armored_public_key.trim();
      if (!t) {
        setError("Title is required.");
        return;
      }
      if (!k) {
        setError("Public key is required.");
        return;
      }
      add.mutate({ title: t, armored_public_key: k });
    },
  });

  const add = useMutation({
    ...gpgKeyAddMutationOptions(apiClient),
    onSuccess: async () => {
      form.reset();
      setError("");
      await queryClient.invalidateQueries({ queryKey: ["gpgKey", "list"] });
      onAdded?.();
    },
    onError: (err: Error) => {
      setError(err.message || "Couldn't add GPG key.");
    },
  });

  <form
    method="post"
    action="#"
    className="mt-6 flex flex-col gap-4 rounded-md border border-border bg-card p-4"
    onSubmit={(e) => {
      e.preventDefault();
      void form.handleSubmit();
    }}
  >
    <form.Field name="title">
      {(field) => (
        <div className="flex flex-col gap-2">
          <Label htmlFor="gpg-key-title">Title</Label>
          <Input
            id="gpg-key-title"
            name="title"
            value={field.state.value}
            disabled={disabled || add.isPending}
            onBlur={field.handleBlur}
            onInput={(e) => field.handleChange((e.currentTarget as HTMLInputElement).value)}
            autoComplete="off"
          />
        </div>
      )}
    </form.Field>
    <form.Field name="armored_public_key">
      {(field) => (
        <div className="flex flex-col gap-2">
          <Label htmlFor="gpg-key-public">Public key</Label>
          <Textarea
            id="gpg-key-public"
            name="armored_public_key"
            rows={8}
            className="font-mono text-[13px]"
            placeholder={
              "-----BEGIN PGP PUBLIC KEY BLOCK-----\n…\n-----END PGP PUBLIC KEY BLOCK-----"
            }
            value={field.state.value}
            disabled={disabled || add.isPending}
            onBlur={field.handleBlur}
            onInput={(e) => field.handleChange((e.currentTarget as HTMLTextAreaElement).value)}
          />
        </div>
      )}
    </form.Field>
    @if (error) {
      <p role="alert" className="text-[14px] text-destructive">
        {error as string}
      </p>;
    }
    <form.Subscribe selector={(s) => s.isSubmitting}>
      {(isSubmitting) => (
        <Button type="submit" disabled={disabled || add.isPending || isSubmitting}>
          {(add.isPending ? "Adding…" : "Add GPG key") as string}
        </Button>
      )}
    </form.Subscribe>
  </form>;
}
