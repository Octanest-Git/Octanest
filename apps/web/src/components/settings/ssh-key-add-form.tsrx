import { sshKeyAddMutationOptions } from "@octanest/api-client";
import { useMutation, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
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
  const [title, setTitle] = useState("");
  const [publicKey, setPublicKey] = useState("");
  const [error, setError] = useState("");

  const add = useMutation({
    ...sshKeyAddMutationOptions(apiClient),
    onSuccess: async () => {
      setTitle("");
      setPublicKey("");
      setError("");
      await queryClient.invalidateQueries({ queryKey: ["sshKey", "list"] });
      onAdded?.();
    },
    onError: (err: Error) => {
      setError(err.message || "Couldn't add SSH key.");
    },
  });

  function submit() {
    if (disabled || add.isPending) return;
    setError("");
    const t = title.trim();
    const k = publicKey.trim();
    if (!t) {
      setError("Title is required.");
      return;
    }
    if (!k) {
      setError("Public key is required.");
      return;
    }
    add.mutate({ title: t, public_key: k });
  }

  <form
    method="post"
    action="#"
    className="mt-6 flex flex-col gap-4 rounded-md border border-border bg-card p-4"
    onSubmit={(e) => {
      e.preventDefault();
      submit();
    }}
  >
    <div className="flex flex-col gap-2">
      <Label htmlFor="ssh-key-title">Title</Label>
      <Input
        id="ssh-key-title"
        name="title"
        value={title as string}
        disabled={disabled || add.isPending}
        onInput={(e) => setTitle((e.currentTarget as HTMLInputElement).value)}
        autoComplete="off"
      />
    </div>
    <div className="flex flex-col gap-2">
      <Label htmlFor="ssh-key-public">Key</Label>
      <Textarea
        id="ssh-key-public"
        name="public_key"
        rows={4}
        className="font-mono text-[13px]"
        placeholder="ssh-ed25519 AAAA… comment"
        value={publicKey as string}
        disabled={disabled || add.isPending}
        onInput={(e) =>
          setPublicKey((e.currentTarget as HTMLTextAreaElement).value)
        }
      />
    </div>
    @if (error) {
      <p role="alert" className="text-[14px] text-destructive">
        {error as string}
      </p>
    }
    <Button
      type="button"
      disabled={disabled || add.isPending}
      onClick={() => submit()}
    >
      {(add.isPending ? "Adding…" : "Add key") as string}
    </Button>
  </form>
}
