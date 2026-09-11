import { createFileRoute, redirect } from "@octanejs/tanstack-router";
import { useEffect, useState } from "octane";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { AuthFormSkeleton } from "@/components/page-skeletons";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { apiClient } from "@/lib/api-client";
import { syncSessionPresenceHint } from "@/lib/session-hint";
import { fetchBootstrapStatus } from "@/lib/ssr-auth";

export const Route = createFileRoute("/setup")({
  component: SetupPage,
  head: () => ({ meta: [{ title: "Set up · Octanest" }] }),
  beforeLoad: async () => {
    let needsSetup: boolean | null = null;
    try {
      const status = await fetchBootstrapStatus();
      if (status.ok) needsSetup = status.data.needs_setup;
    } catch {
      /* API unreachable — client effect still gates */
    }
    if (needsSetup === false) {
      throw redirect({ href: "/" });
    }
  },
});

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const UNAVAILABLE_ERROR =
  "Setup is no longer available. Sign in or create an account instead.";

/** Exported for AUTH-07 integration tests (wizard Switch + CTA). */
export function SetupPage() @{
  const [ready, setReady] = useState(false);
  const [email, setEmail] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [allowSignup, setAllowSignup] = useState(false);
  const [error, setError] = useState("");
  const [pending, setPending] = useState(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const status = await apiClient.auth.bootstrapStatus();
        if (cancelled) return;
        if (!status.ok) {
          setError(NETWORK_ERROR);
          setReady(true);
          return;
        }
        if (!status.data.needs_setup) {
          window.location.assign("/");
          return;
        }
        setReady(true);
      } catch {
        if (cancelled) return;
        setError(NETWORK_ERROR);
        setReady(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

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
    support="Create the system admin account. This is a one-time step for an empty database."
  >
    @if (error) {
      <AuthErrorBanner message={error} />
    }

    @if (!ready) {
      <AuthFormSkeleton fields={4} />
    }

    @if (ready) {
      <form
        className="flex flex-col gap-4"
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
        <Button type="submit" className="w-full" disabled={pending}>
          {(pending ? "Working…" : "Create system admin") as string}
        </Button>
      </form>
    }
  </AuthShell>
}
