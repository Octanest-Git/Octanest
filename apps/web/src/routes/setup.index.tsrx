import { createFileRoute, redirect, useLoaderData } from "@octanejs/tanstack-router";
import type { ProviderMode } from "@octanest/api-client";
import { useState } from "octane";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { apiClient } from "@/lib/api-client";
import { syncSessionPresenceHint } from "@/lib/session-hint";
import { fetchBootstrapStatus } from "@/lib/ssr-auth";

export type SetupLoaderData = {
  loadError: string;
};

export const Route = createFileRoute("/setup/")({
  component: SetupPage,
  head: () => ({ meta: [{ title: "Set up · Octanest" }] }),
  beforeLoad: async () => {
    let needsSetup: boolean | null = null;
    try {
      const status = await fetchBootstrapStatus();
      if (status.ok) needsSetup = status.data.needs_setup;
    } catch {
      /* API unreachable — loader still renders form */
    }
    if (needsSetup === false) {
      throw redirect({ href: "/" });
    }
  },
  loader: async (): Promise<SetupLoaderData> => {
    try {
      const status = await fetchBootstrapStatus();
      if (!status.ok) {
        return {
          loadError: "Can't reach Octanest. Check your connection and try again.",
        };
      }
      if (!status.data.needs_setup) {
        throw redirect({ href: "/" });
      }
      return { loadError: "" };
    } catch (e) {
      if (e && typeof e === "object" && "href" in e) throw e;
      return {
        loadError: "Can't reach Octanest. Check your connection and try again.",
      };
    }
  },
});

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const UNAVAILABLE_ERROR =
  "Setup is no longer available. Sign in or create an account instead.";

const PROVIDER_OPTIONS: { value: ProviderMode; label: string }[] = [
  { value: "local", label: "Local accounts" },
  { value: "workos", label: "WorkOS" },
  { value: "oidc", label: "OIDC / SSO" },
];

/** Exported for AUTH-07 integration tests (wizard Switch + CTA). */
export function SetupPage() @{
  const data = useLoaderData({ from: "/setup/" }) as SetupLoaderData | undefined;
  const [email, setEmail] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [allowSignup, setAllowSignup] = useState(false);
  const [providerMode, setProviderMode] = useState<ProviderMode>("local");
  const [oidcIssuer, setOidcIssuer] = useState("http://127.0.0.1:9090/default");
  const [oidcClientId, setOidcClientId] = useState("octanest-dev");
  const [workosClientId, setWorkosClientId] = useState("");
  const [error, setError] = useState(data?.loadError ?? "");
  const [pending, setPending] = useState(false);

  async function submit() {
    setError("");
    if (password !== confirm) {
      setError("Passwords do not match. Fix the highlighted fields and try again.");
      return;
    }
    setPending(true);
    try {
      const res = await apiClient.auth.bootstrapSetup({
        email: email.trim(),
        username: username.trim(),
        password,
        allow_signup: allowSignup,
        provider_mode: providerMode,
        oidc_issuer: providerMode === "oidc" ? oidcIssuer.trim() || null : null,
        oidc_client_id:
          providerMode === "oidc" ? oidcClientId.trim() || null : null,
        workos_client_id:
          providerMode === "workos" ? workosClientId.trim() || null : null,
      });
      if (!res.ok) {
        const code = res.error.code;
        if (code === "auth.setup_unavailable") setError(UNAVAILABLE_ERROR);
        else if (code === "auth.reserved_username") {
          setError("That username is reserved. Choose a different username.");
        } else if (
          code === "auth.invalid_username" ||
          code === "auth.invalid_email" ||
          code === "auth.weak_password" ||
          code === "rpc.bad_input"
        ) {
          setError(
            `${res.error.message}. Fix the highlighted fields and try again.`,
          );
        } else {
          setError(NETWORK_ERROR);
        }
        setPending(false);
        return;
      }
      syncSessionPresenceHint(true);
      window.location.assign("/");
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  <AuthShell
    title="Set up this instance"
    support="Create the system admin account and choose how people will sign in."
  >
    @if (error) {
      <AuthErrorBanner message={error} />
    }

    <form
      className="flex flex-col gap-4"
      method="post"
      action="#"
      onSubmit={(e) => {
        e.preventDefault();
        void submit();
      }}
    >
      <div className="flex flex-col gap-2">
        <Label htmlFor="setup-email">Email</Label>
        <Input
          id="setup-email"
          name="email"
          type="email"
          autoComplete="email"
          value={email}
          onInput={(ev) => setEmail((ev.currentTarget as HTMLInputElement).value)}
          required
        />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor="setup-username">Username</Label>
        <Input
          id="setup-username"
          name="username"
          autoComplete="username"
          value={username}
          onInput={(ev) =>
            setUsername((ev.currentTarget as HTMLInputElement).value)
          }
          required
        />
        <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
          1–39 characters; letters, numbers, and hyphens. Can’t start or end with
          a hyphen.
        </p>
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor="setup-password">Password</Label>
        <Input
          id="setup-password"
          name="password"
          type="password"
          autoComplete="new-password"
          value={password}
          onInput={(ev) =>
            setPassword((ev.currentTarget as HTMLInputElement).value)
          }
          required
        />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor="setup-confirm">Confirm password</Label>
        <Input
          id="setup-confirm"
          name="confirm"
          type="password"
          autoComplete="new-password"
          value={confirm}
          onInput={(ev) =>
            setConfirm((ev.currentTarget as HTMLInputElement).value)
          }
          required
        />
      </div>

      <div className="flex flex-col gap-2">
        <Label htmlFor="setup-provider">Auth stack</Label>
        <select
          id="setup-provider"
          className="flex h-11 w-full rounded-md border border-border bg-background px-3 text-[16px] text-foreground"
          value={providerMode}
          onChange={(e) =>
            setProviderMode(
              (e.currentTarget as HTMLSelectElement).value as ProviderMode,
            )
          }
        >
          @for (const opt of PROVIDER_OPTIONS; key opt.value) {
            <option value={opt.value}>{opt.label as string}</option>
          }
        </select>
        <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
          Secrets (OIDC client secret, WorkOS API key) stay in environment
          variables — only public IDs are stored here.
        </p>
      </div>

      {providerMode === "workos" ? (
        <div className="flex flex-col gap-2">
          <Label htmlFor="setup-workos-client-id">WorkOS client ID</Label>
          <Input
            id="setup-workos-client-id"
            value={workosClientId}
            onInput={(ev) =>
              setWorkosClientId((ev.currentTarget as HTMLInputElement).value)
            }
            autoComplete="off"
          />
        </div>
      ) : null}

      {providerMode === "oidc" ? (
        <div className="flex flex-col gap-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="setup-oidc-issuer">OIDC issuer</Label>
            <Input
              id="setup-oidc-issuer"
              value={oidcIssuer}
              onInput={(ev) =>
                setOidcIssuer((ev.currentTarget as HTMLInputElement).value)
              }
              autoComplete="off"
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="setup-oidc-client-id">OIDC client ID</Label>
            <Input
              id="setup-oidc-client-id"
              value={oidcClientId}
              onInput={(ev) =>
                setOidcClientId((ev.currentTarget as HTMLInputElement).value)
              }
              autoComplete="off"
            />
          </div>
        </div>
      ) : null}

      <div className="flex flex-col gap-2">
        <div className="flex min-h-11 items-center gap-2">
          <Switch
            id="setup-allow-signup"
            checked={allowSignup}
            onCheckedChange={(checked) => setAllowSignup(checked === true)}
          />
          <Label htmlFor="setup-allow-signup">Allow open signup</Label>
        </div>
        <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
          When off, new accounts can’t self-register. You can change this later in
          Auth settings.
        </p>
      </div>
      <Button
        type="button"
        className="w-full"
        disabled={pending}
        onClick={() => void submit()}
      >
        {(pending ? "Working…" : "Create system admin") as string}
      </Button>
    </form>
  </AuthShell>
}
