import {
  createFileRoute,
  isNotFound,
  notFound,
  redirect,
  useLoaderData,
} from "@octanejs/tanstack-router";
import type { ProviderMode } from "@octanest/api-client";
import { useState } from "octane";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { apiClient } from "@/lib/api-client";
import { readReturnToFromLocation, safeReturnTo } from "@/lib/return-to";
import { fetchProviderConfig, fetchSessionMe } from "@/lib/ssr-auth";

export type SignupLoaderData = {
  mode: ProviderMode;
  loadError: string;
};

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const TAKEN_ERROR =
  "That email or username is already taken. Try another or sign in.";
const RESERVED_ERROR = "That username is reserved. Choose a different username.";
const SETUP_ERROR = "This instance needs setup first. Redirecting…";

export const Route = createFileRoute("/signup")({
  component: SignupPage,
  head: () => ({ meta: [{ title: "Sign up · Octanest" }] }),
  beforeLoad: async () => {
    // D-06 / UI-SPEC: closed signup is SSR 404 — never soft AuthShell.
    try {
      const me = await fetchSessionMe();
      if (me.ok) {
        throw redirect({ href: "/" });
      }
    } catch (e) {
      if (e && typeof e === "object" && "href" in e) throw e;
    }
    try {
      const cfg = await fetchProviderConfig();
      if (cfg.ok && cfg.data.allow_signup === false) {
        throw notFound();
      }
    } catch (e) {
      if (isNotFound(e)) throw e;
      /* API unreachable — leave to loader fallback */
    }
  },
  loader: async (): Promise<SignupLoaderData> => {
    try {
      const cfg = await fetchProviderConfig();
      if (!cfg.ok) {
        return { mode: "local", loadError: NETWORK_ERROR };
      }
      return { mode: cfg.data.mode, loadError: "" };
    } catch {
      return { mode: "local", loadError: NETWORK_ERROR };
    }
  },
});

/** Exported for AUTH-05 integration tests (no invite UI). */
export function SignupPage() @{
  const data = useLoaderData({ from: "/signup" }) as SignupLoaderData | undefined;
  const mode = data?.mode ?? "local";
  const loadError = data?.loadError ?? "";
  const [email, setEmail] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [pending, setPending] = useState(false);

  const returnTo = readReturnToFromLocation();
  const loginHref = `/login?returnTo=${encodeURIComponent(returnTo)}`;

  function startSso(kind: "workos" | "oidc") {
    const path =
      kind === "workos" ? "/api/auth/workos/start" : "/api/auth/oidc/start";
    window.location.assign(`${path}?returnTo=${encodeURIComponent(returnTo)}`);
  }

  async function submitLocal() {
    setError("");
    if (password !== confirm) {
      setError("Passwords do not match. Fix the highlighted fields and try again.");
      return;
    }
    setPending(true);
    try {
      const res = await apiClient.auth.signup({
        email: email.trim(),
        username: username.trim(),
        password,
      });
      if (!res.ok) {
        const code = res.error.code;
        if (code === "auth.taken") setError(TAKEN_ERROR);
        else if (code === "auth.reserved_username") setError(RESERVED_ERROR);
        else if (code === "auth.setup_required") {
          setError(SETUP_ERROR);
          window.location.assign("/setup");
          return;
        } else if (
          code === "auth.invalid_username" ||
          code === "auth.invalid_email" ||
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
      window.location.assign(safeReturnTo(returnTo));
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  const support =
    mode === "workos" || mode === "oidc"
      ? "You’ll finish signup with your identity provider."
      : "Email, username, and password — GitHub-shaped handles.";

  <AuthShell title="Create your account" support={support}>
    @if (loadError) {
      <AuthErrorBanner message={loadError} />
    }
    @if (error) {
      <AuthErrorBanner message={error} />
    }

    @if (mode === "local") {
      <form
        className="flex flex-col gap-4"
        method="post"
        action="#"
        onSubmit={(e) => {
          e.preventDefault();
          void submitLocal();
        }}
      >
        <div className="flex flex-col gap-2">
          <Label htmlFor="signup-email">Email</Label>
          <Input
            id="signup-email"
            name="email"
            type="email"
            autoComplete="email"
            value={email}
            onInput={(ev) => setEmail((ev.currentTarget as HTMLInputElement).value)}
            required
          />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="signup-username">Username</Label>
          <Input
            id="signup-username"
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
          <Label htmlFor="signup-password">Password</Label>
          <Input
            id="signup-password"
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
          <Label htmlFor="signup-confirm">Confirm password</Label>
          <Input
            id="signup-confirm"
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
        <Button
          type="button"
          className="w-full"
          disabled={pending}
          onClick={() => void submitLocal()}
        >
          {(pending ? "Working…" : "Create account") as string}
        </Button>
      </form>
    }

    @if (mode === "workos") {
      <Button type="button" className="w-full" onClick={() => startSso("workos")}>
        Continue with WorkOS
      </Button>
    }

    @if (mode === "oidc") {
      <Button type="button" className="w-full" onClick={() => startSso("oidc")}>
        Continue with SSO
      </Button>
    }

    <p className="text-[14px] text-muted-foreground">
      Already have an account?{" "}
      <a
        href={loginHref as string}
        className="font-normal text-foreground underline-offset-4 hover:underline"
      >
        Sign in
      </a>
    </p>
  </AuthShell>
}
