import { createFileRoute } from "@octanejs/tanstack-router";
import type { ProviderMode } from "@octanest/api-client";
import { useEffect, useRef, useState } from "octane";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { InputOtp } from "@/components/ui/input-otp";
import { Label } from "@/components/ui/label";
import { apiClient } from "@/lib/api-client";
import { readReturnToFromLocation, safeReturnTo } from "@/lib/return-to";

export const Route = createFileRoute("/reset-password")({
  component: ResetPasswordPage,
  head: () => ({
    meta: [
      { title: "Reset password · Octanest" },
      { name: "referrer", content: "no-referrer" },
    ],
  }),
});

const REQUEST_SUPPORT =
  "Enter your account email. If it matches a local-password account, we’ll send a link and an 8-digit code.";
const REDEEM_SUPPORT =
  "Enter the 8-digit code from your email (or continue from your reset link), then set a new password.";
const SSO_MODE_BODY = "Password reset is managed by your identity provider.";
const SUCCESS_HEADING = "Check your email";
const SUCCESS_BODY =
  "If an account exists for that email, we sent password reset instructions. Check your inbox and spam folder.";
const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";
const MISMATCH =
  "Passwords don’t match. Fix the highlighted fields and try again.";
const TOO_SHORT = "Password must be at least 8 characters.";
const INVALID_EXPIRED =
  "That code or link is invalid or expired. Request a new reset email and try again.";
const SSO_ONLY =
  "This account signs in with SSO. Reset your password with your identity provider.";
const PASSWORD_HELPER = "At least 8 characters.";

function readTokenFromLocation(): string | null {
  if (typeof window === "undefined") return null;
  const raw = new URLSearchParams(window.location.search).get("token");
  if (!raw) return null;
  const trimmed = raw.trim();
  return trimmed.length > 0 ? trimmed : null;
}

type View =
  | { kind: "loading" }
  | { kind: "sso"; mode: "workos" | "oidc" }
  | { kind: "request" }
  | { kind: "request-sent" }
  | { kind: "redeem"; token: string | null };

function ResetPasswordPage() {
  const [view, setView] = useState<View>({ kind: "loading" });
  const [email, setEmail] = useState("");
  const [code, setCode] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [pending, setPending] = useState(false);
  const [mismatchFields, setMismatchFields] = useState(false);
  const submitLock = useRef(false);
  const passwordRef = useRef(password);
  const confirmRef = useRef(confirm);
  passwordRef.current = password;
  confirmRef.current = confirm;

  useEffect(() => {
    let cancelled = false;
    const token = readTokenFromLocation();
    (async () => {
      try {
        const cfg = await apiClient.auth.providerConfig();
        if (cancelled) return;
        if (!cfg.ok) {
          setError(NETWORK_ERROR);
          setView(token ? { kind: "redeem", token } : { kind: "request" });
          return;
        }
        const mode: ProviderMode = cfg.data.mode;
        if (mode === "workos" || mode === "oidc") {
          setView({ kind: "sso", mode });
          return;
        }
        setView(token ? { kind: "redeem", token } : { kind: "request" });
      } catch {
        if (cancelled) return;
        setError(NETWORK_ERROR);
        setView(token ? { kind: "redeem", token } : { kind: "request" });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  function startSso(kind: "workos" | "oidc") {
    const returnTo = readReturnToFromLocation();
    const path =
      kind === "workos" ? "/api/auth/workos/start" : "/api/auth/oidc/start";
    window.location.assign(`${path}?returnTo=${encodeURIComponent(returnTo)}`);
  }

  async function sendReset() {
    if (pending) return;
    const trimmed = email.trim();
    if (!trimmed) return;
    setPending(true);
    setError("");
    try {
      const res = await apiClient.auth.requestPasswordReset({ email: trimmed });
      if (!res.ok) {
        // Anti-enumeration: never show not-found; only network/validation surface.
        if (
          res.error.code === "rpc.bad_input" ||
          res.error.code === "auth.invalid_email"
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
      setPending(false);
      setView({ kind: "request-sent" });
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  function mapRedeemError(code: string, message: string): string {
    if (code === "auth.sso_only") return SSO_ONLY;
    if (code === "auth.weak_password") return TOO_SHORT;
    if (code === "auth.invalid_token") return INVALID_EXPIRED;
    return message || INVALID_EXPIRED;
  }

  async function redeem(opts: {
    nextCode?: string;
    useToken?: boolean;
  }) {
    if (submitLock.current || pending || view.kind !== "redeem") return;
    const pwd = passwordRef.current;
    const conf = confirmRef.current;
    if (pwd.length < 8) {
      setError(TOO_SHORT);
      setMismatchFields(false);
      return;
    }
    if (pwd !== conf) {
      setError(MISMATCH);
      setMismatchFields(true);
      return;
    }
    const token = opts.useToken === false ? null : view.token;
    const nextCode = (opts.nextCode ?? code).trim();
    if (!token && (nextCode.length !== 8 || !/^\d{8}$/.test(nextCode))) {
      setError(INVALID_EXPIRED);
      return;
    }

    submitLock.current = true;
    setPending(true);
    setError("");
    setMismatchFields(false);
    try {
      const res = await apiClient.auth.resetPassword({
        token: token || null,
        code: nextCode.length === 8 ? nextCode : null,
        password: pwd,
      });
      if (!res.ok) {
        setError(mapRedeemError(res.error.code, res.error.message));
        setPending(false);
        submitLock.current = false;
        return;
      }
      window.location.assign(safeReturnTo(readReturnToFromLocation()));
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
      submitLock.current = false;
    }
  }

  if (view.kind === "loading") {
    return (
      <AuthShell title="Reset your password" support="Loading…">
        <p className="text-[16px] text-muted-foreground">Working…</p>
      </AuthShell>
    );
  }

  if (view.kind === "sso") {
    return (
      <AuthShell title="Reset your password" support={SSO_MODE_BODY}>
        {error ? <AuthErrorBanner message={error} /> : null}
        <Button
          type="button"
          className="w-full"
          onClick={() => startSso(view.mode)}
        >
          {view.mode === "workos" ? "Continue with WorkOS" : "Continue with SSO"}
        </Button>
        <p className="text-[14px] text-muted-foreground">
          Remembered it?{" "}
          <a
            href="/login"
            className="font-normal text-foreground underline-offset-4 hover:underline"
          >
            Sign in
          </a>
        </p>
      </AuthShell>
    );
  }

  if (view.kind === "request" || view.kind === "request-sent") {
    return (
      <AuthShell title="Reset your password" support={REQUEST_SUPPORT}>
        {error ? <AuthErrorBanner message={error} /> : null}

        {view.kind === "request-sent" ? (
          <div className="flex max-w-md flex-col gap-2" role="status">
            <p className="text-[16px] font-normal leading-[1.5] text-foreground">
              {SUCCESS_HEADING}
            </p>
            <p className="text-[16px] font-normal leading-[1.5] text-muted-foreground">
              {SUCCESS_BODY}
            </p>
            <Button
              type="button"
              className="mt-2 w-full"
              onClick={() => {
                setError("");
                setView({ kind: "redeem", token: null });
              }}
            >
              Enter reset code
            </Button>
          </div>
        ) : (
          <form
            className="flex flex-col gap-4"
            onSubmit={(e) => {
              e.preventDefault();
              void sendReset();
            }}
          >
            <div className="flex flex-col gap-2">
              <Label htmlFor="reset-email">Email</Label>
              <Input
                id="reset-email"
                name="email"
                type="email"
                autoComplete="email"
                value={email}
                onInput={(ev) =>
                  setEmail((ev.currentTarget as HTMLInputElement).value)
                }
                required
                disabled={pending}
              />
            </div>
            <Button type="submit" className="w-full" disabled={pending}>
              {pending ? "Working…" : "Send reset email"}
            </Button>
          </form>
        )}

        {view.kind === "request" ? (
          <p className="text-[14px] text-muted-foreground">
            Already have a code?{" "}
            <button
              type="button"
              className="font-normal text-foreground underline-offset-4 hover:underline"
              onClick={() => {
                setError("");
                setView({ kind: "redeem", token: null });
              }}
            >
              Enter it here
            </button>
          </p>
        ) : null}

        <p className="text-[14px] text-muted-foreground">
          Remembered it?{" "}
          <a
            href="/login"
            className="font-normal text-foreground underline-offset-4 hover:underline"
          >
            Sign in
          </a>
        </p>
      </AuthShell>
    );
  }

  const canSubmit =
    password.length >= 8 &&
    password === confirm &&
    (Boolean(view.token) || code.length === 8);

  return (
    <AuthShell title="Choose a new password" support={REDEEM_SUPPORT}>
      {error ? <AuthErrorBanner message={error} /> : null}

      <form
        className="flex flex-col gap-4"
        onSubmit={(e) => {
          e.preventDefault();
          void redeem({});
        }}
      >
        <div className="flex flex-col gap-2">
          <Label htmlFor="reset-code">Reset code</Label>
          <InputOtp
            id="reset-code"
            name="code"
            value={code}
            onChange={(next) => {
              setCode(next);
              if (error) setError("");
            }}
            onComplete={(next) => {
              void redeem({ nextCode: next });
            }}
            disabled={pending}
            aria-label="Reset code"
            aria-invalid={Boolean(error)}
          />
        </div>

        <div className="flex flex-col gap-2">
          <Label htmlFor="reset-password">New password</Label>
          <Input
            id="reset-password"
            name="password"
            type="password"
            autoComplete="new-password"
            value={password}
            onInput={(ev) => {
              setPassword((ev.currentTarget as HTMLInputElement).value);
              if (mismatchFields) setMismatchFields(false);
              if (error) setError("");
            }}
            required
            disabled={pending}
            aria-invalid={mismatchFields || error === TOO_SHORT}
            aria-describedby="reset-password-helper"
          />
          <p
            id="reset-password-helper"
            className="text-[14px] font-normal leading-[1.4] text-muted-foreground"
          >
            {PASSWORD_HELPER}
          </p>
        </div>

        <div className="flex flex-col gap-2">
          <Label htmlFor="reset-confirm">Confirm password</Label>
          <Input
            id="reset-confirm"
            name="confirm"
            type="password"
            autoComplete="new-password"
            value={confirm}
            onInput={(ev) => {
              setConfirm((ev.currentTarget as HTMLInputElement).value);
              if (mismatchFields) setMismatchFields(false);
              if (error) setError("");
            }}
            required
            disabled={pending}
            aria-invalid={mismatchFields}
          />
        </div>

        <Button
          type="submit"
          className="w-full"
          disabled={pending || !canSubmit}
        >
          {pending ? "Working…" : "Update password"}
        </Button>
      </form>

      <p className="text-[14px] text-muted-foreground">
        Remembered it?{" "}
        <a
          href="/login"
          className="font-normal text-foreground underline-offset-4 hover:underline"
        >
          Sign in
        </a>
      </p>
    </AuthShell>
  );
}
