import { createFileRoute, redirect } from "@octanejs/tanstack-router";
import { useEffect, useState } from "octane";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { AuthFormSkeleton } from "@/components/page-skeletons";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { apiClient } from "@/lib/api-client";
import { readReturnToFromLocation, safeReturnTo } from "@/lib/return-to";
import { fetchSessionMe } from "@/lib/ssr-auth";

const DEFAULT_USERNAME = "system-administrator";
const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const DEFAULT_USERNAME_ERROR =
  "Choose a username other than the default system-administrator.";

export const Route = createFileRoute("/setup/credentials")({
  component: CredentialsPage,
  head: () => ({ meta: [{ title: "Confirm admin · Octanest" }] }),
  beforeLoad: async () => {
    let mustChange: boolean | null = null;
    try {
      const me = await fetchSessionMe();
      if (me.ok) mustChange = me.data.must_change_credentials;
      else mustChange = false;
    } catch {
      /* unreachable — client effect still gates */
    }
    if (mustChange === false) {
      throw redirect({ href: "/" });
    }
  },
});

/** Exported for AUTH-06 integration tests (Keep password Switch + default username). */
export function CredentialsPage() @{
  const [ready, setReady] = useState(false);
  const [email, setEmail] = useState("");
  const [username, setUsername] = useState("");
  const [keepPassword, setKeepPassword] = useState(true);
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [pending, setPending] = useState(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const me = await apiClient.auth.me();
        if (cancelled) return;
        if (!me.ok) {
          window.location.assign(
            `/login?returnTo=${encodeURIComponent("/setup/credentials")}`,
          );
          return;
        }
        if (!me.data.must_change_credentials) {
          window.location.assign(safeReturnTo(readReturnToFromLocation()));
          return;
        }
        setEmail(me.data.email);
        setUsername(me.data.username);
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
    const nextUsername = username.trim();
    if (nextUsername.toLowerCase() === DEFAULT_USERNAME) {
      setError(DEFAULT_USERNAME_ERROR);
      return;
    }
    if (!keepPassword) {
      if (password.length < 8) {
        setError("Password must be at least 8 characters. Fix the highlighted fields and try again.");
        return;
      }
      if (password !== confirm) {
        setError("Passwords do not match. Fix the highlighted fields and try again.");
        return;
      }
    }
    setPending(true);
    try {
      const res = await apiClient.auth.confirmAdminCredentials({
        username: nextUsername,
        email: email.trim() || null,
        password: keepPassword ? null : password,
        keep_password: keepPassword,
      });
      if (!res.ok) {
        const code = res.error.code;
        if (code === "auth.invalid_username") {
          setError(DEFAULT_USERNAME_ERROR);
        } else if (code === "auth.taken") {
          setError("That email or username is already taken. Try another.");
        } else if (
          code === "auth.invalid_email" ||
          code === "auth.weak_password" ||
          code === "rpc.bad_input"
        ) {
          setError(
            `${res.error.message}. Fix the highlighted fields and try again.`,
          );
        } else if (code === "auth.credentials_not_required") {
          window.location.assign(safeReturnTo(readReturnToFromLocation()));
          return;
        } else if (code === "auth.unauthenticated") {
          window.location.assign(
            `/login?returnTo=${encodeURIComponent("/setup/credentials")}`,
          );
          return;
        } else {
          setError(NETWORK_ERROR);
        }
        setPending(false);
        return;
      }
      window.location.assign(safeReturnTo(readReturnToFromLocation()));
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  <AuthShell
    title="Confirm admin account"
    support="Review your admin email and username. Change any still-default values before continuing."
  >
    @if (error) {
      <AuthErrorBanner message={error} />
    }

    @if (!ready) {
      <AuthFormSkeleton fields={3} />
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
          <Label htmlFor="credentials-email">Email</Label>
          <Input
            id="credentials-email"
            name="email"
            type="email"
            autoComplete="email"
            value={email}
            onInput={(ev) => setEmail((ev.currentTarget as HTMLInputElement).value)}
            required
          />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="credentials-username">Username</Label>
          <Input
            id="credentials-username"
            name="username"
            autoComplete="username"
            value={username}
            onInput={(ev) =>
              setUsername((ev.currentTarget as HTMLInputElement).value)
            }
            required
          />
          <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
            Change the default system-administrator username before continuing.
          </p>
        </div>
        <div className="flex flex-col gap-2">
          <div className="flex min-h-11 items-center gap-2">
            <Switch
              id="credentials-keep-password"
              checked={keepPassword}
              onCheckedChange={(checked) => setKeepPassword(checked === true)}
            />
            <Label htmlFor="credentials-keep-password">Keep current password</Label>
          </div>
        </div>
        @if (!keepPassword) {
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="credentials-password">New password</Label>
              <Input
                id="credentials-password"
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
              <Label htmlFor="credentials-confirm">Confirm password</Label>
              <Input
                id="credentials-confirm"
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
          </div>
        }
        <Button type="submit" className="w-full" disabled={pending}>
          {(pending ? "Working…" : "Save and continue") as string}
        </Button>
      </form>
    }
  </AuthShell>
}
