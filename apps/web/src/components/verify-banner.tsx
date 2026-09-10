import { useEffect, useState } from "octane";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";

const BANNER_MESSAGE =
  "Verify your email to unlock creating repositories and other privileged actions.";
const RESEND_SUCCESS = "Email sent.";
const RESEND_RATE_LIMITED = "Please wait before requesting another email.";
const RESEND_COOLDOWN_HELPER = "Resend available shortly";
const NETWORK_ERROR =
  "Can't reach Octanest. Check your connection and try again.";
const RESEND_COOLDOWN_SECS = 60;

/**
 * Persistent chrome strip for signed-in unverified sessions (D-11, D-13).
 * Mounted under SiteHeader — does not steal focus on navigation.
 */
export function VerifyBanner() {
  const [visible, setVisible] = useState(false);
  const [ready, setReady] = useState(false);
  const [resendPending, setResendPending] = useState(false);
  const [note, setNote] = useState("");
  const [error, setError] = useState("");
  const [cooldownUntil, setCooldownUntil] = useState(0);
  const [cooldownLeft, setCooldownLeft] = useState(0);
  const [alertMode, setAlertMode] = useState(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await apiClient.auth.me();
        if (cancelled) return;
        if (res.ok && res.data.email_verified === false) {
          setVisible(true);
        } else {
          setVisible(false);
        }
      } catch {
        if (cancelled) return;
        setVisible(false);
      } finally {
        if (!cancelled) setReady(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (cooldownUntil <= 0) {
      setCooldownLeft(0);
      return;
    }
    const tick = () => {
      const left = Math.max(0, Math.ceil((cooldownUntil - Date.now()) / 1000));
      setCooldownLeft(left);
      if (left <= 0) setCooldownUntil(0);
    };
    tick();
    const id = window.setInterval(tick, 250);
    return () => window.clearInterval(id);
  }, [cooldownUntil]);

  async function resend() {
    if (resendPending || cooldownLeft > 0) return;
    setResendPending(true);
    setNote("");
    setError("");
    setAlertMode(false);
    try {
      const res = await apiClient.auth.resendVerify();
      if (!res.ok) {
        if (res.error.code === "auth.rate_limited") {
          setError(RESEND_RATE_LIMITED);
          setCooldownUntil(Date.now() + RESEND_COOLDOWN_SECS * 1000);
          setAlertMode(true);
        } else {
          setError(res.error.message || NETWORK_ERROR);
          setAlertMode(true);
        }
        setResendPending(false);
        return;
      }
      setNote(RESEND_SUCCESS);
      setCooldownUntil(Date.now() + RESEND_COOLDOWN_SECS * 1000);
      setResendPending(false);
    } catch {
      setError(NETWORK_ERROR);
      setAlertMode(true);
      setResendPending(false);
    }
  }

  if (!ready || !visible) return null;

  const cooldownHelper =
    cooldownLeft > 0
      ? `You can resend in ${cooldownLeft}s`
      : error === RESEND_RATE_LIMITED
        ? RESEND_COOLDOWN_HELPER
        : "";

  return (
    <div
      role={alertMode ? "alert" : "status"}
      className="oct-verify-banner border-b border-border bg-card px-4 py-2"
    >
      <div className="mx-auto flex w-full max-w-5xl flex-wrap items-center gap-x-6 gap-y-2">
        <p className="min-w-0 flex-1 text-[14px] leading-[1.4] text-foreground break-words">
          {BANNER_MESSAGE}
        </p>
        <div className="flex flex-wrap items-center gap-3">
          <Button
            type="button"
            variant="ghost"
            className="h-11 px-2"
            disabled={resendPending || cooldownLeft > 0}
            onClick={() => {
              void resend();
            }}
          >
            {resendPending ? "Working…" : "Resend email"}
          </Button>
          <a
            href="/verify"
            className="inline-flex h-11 items-center text-[14px] font-normal text-foreground underline underline-offset-4"
          >
            Enter code
          </a>
        </div>
        {note ? (
          <p className="w-full text-[14px] text-muted-foreground">{note}</p>
        ) : null}
        {cooldownHelper ? (
          <p className="w-full text-[14px] text-muted-foreground">{cooldownHelper}</p>
        ) : null}
        {error ? (
          <p className="w-full text-[14px] text-destructive/90">{error}</p>
        ) : null}
      </div>
    </div>
  );
}
