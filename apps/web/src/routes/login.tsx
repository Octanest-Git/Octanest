import { createFileRoute } from "@octanejs/tanstack-router";
import type { ProviderMode } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { apiClient } from "@/lib/api-client";
import { readReturnToFromLocation, safeReturnTo } from "@/lib/return-to";

export const Route = createFileRoute("/login")({
  component: LoginPage,
  head: () => ({ meta: [{ title: "Sign in · Octanest" }] }),
});

const LOGIN_ERROR =
  "Incorrect email/username or password. Check your details and try again.";
const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

function LoginPage() {
  const [mode, setMode] = useState<ProviderMode | null>(null);
  const [loadError, setLoadError] = useState("");
  const [identifier, setIdentifier] = useState("");
  const [password, setPassword] = useState("");
  const [rememberMe, setRememberMe] = useState(false);
  const [error, setError] = useState("");
  const [pending, setPending] = useState(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const me = await apiClient.auth.me();
        if (cancelled) return;
        if (me.ok) {
          window.location.assign(safeReturnTo(readReturnToFromLocation()));
          return;
        }
      } catch {
        /* anonymous */
      }
      try {
        const cfg = await apiClient.auth.providerConfig();
        if (cancelled) return;
        if (!cfg.ok) {
          setLoadError(NETWORK_ERROR);
          setMode("local");
          return;
        }
        setMode(cfg.data.mode);
      } catch {
        if (cancelled) return;
        setLoadError(NETWORK_ERROR);
        setMode("local");
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const returnTo = readReturnToFromLocation();
  const signupHref = `/signup?returnTo=${encodeURIComponent(returnTo)}`;

  function startSso(kind: "workos" | "oidc") {
    const path =
      kind === "workos" ? "/api/auth/workos/start" : "/api/auth/oidc/start";
    window.location.assign(`${path}?returnTo=${encodeURIComponent(returnTo)}`);
  }

  async function submitLocal() {
    setError("");
    setPending(true);
    try {
      const res = await apiClient.auth.login({
        identifier: identifier.trim(),
        password,
        remember_me: rememberMe,
      });
      if (!res.ok) {
        setError(LOGIN_ERROR);
        setPending(false);
        return;
      }
      window.location.assign(returnTo);
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  const support =
    mode === "workos"
      ? "Continue with WorkOS to access this Octanest instance."
      : mode === "oidc"
        ? "Continue with your organization’s SSO to access this instance."
        : "Use your email or username to continue.";

  return (
    <AuthShell title="Sign in" support={mode ? support : "Loading…"}>
      {loadError ? <AuthErrorBanner message={loadError} /> : null}
      {error ? <AuthErrorBanner message={error} /> : null}

      {mode === "local" ? (
        <form
          className="flex flex-col gap-4"
          onSubmit={(e) => {
            e.preventDefault();
            void submitLocal();
          }}
        >
          <div className="flex flex-col gap-2">
            <Label htmlFor="login-identifier">Email or username</Label>
            <Input
              id="login-identifier"
              name="identifier"
              autoComplete="username"
              placeholder="Email or username"
              value={identifier}
              onInput={(ev) =>
                setIdentifier((ev.currentTarget as HTMLInputElement).value)
              }
              required
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="login-password">Password</Label>
            <Input
              id="login-password"
              name="password"
              type="password"
              autoComplete="current-password"
              value={password}
              onInput={(ev) =>
                setPassword((ev.currentTarget as HTMLInputElement).value)
              }
              required
            />
          </div>
          <div className="flex min-h-11 items-center gap-3">
            <Checkbox
              id="login-remember"
              checked={rememberMe}
              onCheckedChange={(checked) => setRememberMe(checked === true)}
            />
            <Label htmlFor="login-remember">Remember me</Label>
          </div>
          <Button type="submit" className="w-full" disabled={pending}>
            {pending ? "Working…" : "Sign in"}
          </Button>
        </form>
      ) : null}

      {mode === "workos" ? (
        <Button type="button" className="w-full" onClick={() => startSso("workos")}>
          Continue with WorkOS
        </Button>
      ) : null}

      {mode === "oidc" ? (
        <Button type="button" className="w-full" onClick={() => startSso("oidc")}>
          Continue with SSO
        </Button>
      ) : null}

      {mode ? (
        <p className="text-[14px] text-muted-foreground">
          New to Octanest?{" "}
          <a
            href={signupHref}
            className="font-normal text-foreground underline-offset-4 hover:underline"
          >
            Create an account
          </a>
        </p>
      ) : null}
    </AuthShell>
  );
}
