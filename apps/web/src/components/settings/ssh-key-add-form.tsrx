import { sshKeyAddMutationOptions } from "@oxidean/api-client";
import { useMutation, useQueryClient } from "@octanejs/tanstack-query";
import { useForm } from "@octanejs/tanstack-form";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { apiClient } from "@/lib/api-client";

type SshKeyAddFormProps = {
  disabled?: boolean;
  onAdded?: () => void;
};

/** Title + public key form — no one-time secret reveal (D-SSH-05). */
export function SshKeyAddForm({ disabled = false, onAdded }: SshKeyAddFormProps) @{
  const queryClient = useQueryClient();
  const [error, setError] = useState("");
  const [canAuthenticate, setCanAuthenticate] = useState(true);
  const [canSign, setCanSign] = useState(true);

  const form = useForm({
    defaultValues: {
      title: "",
      public_key: "",
    },
    onSubmit: ({ value }) => {
      if (disabled || add.isPending) return;
      setError("");
      const t = value.title.trim();
      const k = value.public_key.trim();
      if (!t) {
        setError("Title is required.");
        return;
      }
      if (!k) {
        setError("Public key is required.");
        return;
      }
      if (!canAuthenticate && !canSign) {
        setError("Enable authentication and/or commit signing.");
        return;
      }
      add.mutate({
        title: t,
        public_key: k,
        can_authenticate: canAuthenticate,
        can_sign: canSign,
      });
    },
  });

  const add = useMutation({
    ...sshKeyAddMutationOptions(apiClient),
    onSuccess: async () => {
      form.reset();
      setError("");
      setCanAuthenticate(true);
      setCanSign(true);
      await queryClient.invalidateQueries({ queryKey: ["sshKey", "list"] });
      onAdded?.();
    },
    onError: (err: Error) => {
      setError(err.message || "Couldn't add SSH key.");
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
          <Label htmlFor="ssh-key-title">Title</Label>
          <Input
            id="ssh-key-title"
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
    <form.Field name="public_key">
      {(field) => (
        <div className="flex flex-col gap-2">
          <Label htmlFor="ssh-key-public">Key</Label>
          <Textarea
            id="ssh-key-public"
            name="public_key"
            rows={4}
            className="font-mono text-[13px]"
            placeholder="ssh-ed25519 AAAA… comment"
            value={field.state.value}
            disabled={disabled || add.isPending}
            onBlur={field.handleBlur}
            onInput={(e) => field.handleChange((e.currentTarget as HTMLTextAreaElement).value)}
          />
        </div>
      )}
    </form.Field>
    <div className="flex flex-col gap-2">
      <p className="text-[14px] font-medium text-foreground">Usage</p>
      <label className="flex cursor-pointer items-start gap-3">
        <Checkbox
          checked={canAuthenticate}
          disabled={disabled || add.isPending}
          aria-label="Authentication"
          onCheckedChange={(checked) => setCanAuthenticate(checked === true)}
        />
        <span className="text-[14px] text-foreground">Authentication (git over SSH)</span>
      </label>
      <label className="flex cursor-pointer items-start gap-3">
        <Checkbox
          checked={canSign}
          disabled={disabled || add.isPending}
          aria-label="Commit signing"
          onCheckedChange={(checked) => setCanSign(checked === true)}
        />
        <span className="text-[14px] text-foreground">Commit signing (SSH signatures)</span>
      </label>
    </div>
    @if (error) {
      <p role="alert" className="text-[14px] text-destructive">
        {error as string}
      </p>;
    }
    <form.Subscribe selector={(s) => s.isSubmitting}>
      {(isSubmitting) => (
        <Button type="submit" disabled={disabled || add.isPending || isSubmitting}>
          {(add.isPending ? "Adding…" : "Add SSH key") as string}
        </Button>
      )}
    </form.Subscribe>
  </form>;
}
