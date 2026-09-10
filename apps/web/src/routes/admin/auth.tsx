import { Check, ChevronDown } from "@octanejs/lucide";
import { createFileRoute } from "@octanejs/tanstack-router";
import type {
  AuthSettingsPublic,
  EmailProviderKind,
  ProviderMode,
  UserPublic,
} from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  SelectIcon,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectList,
  SelectPopup,
  SelectPortal,
  SelectPositioner,
  SelectRoot,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { apiClient } from "@/lib/api-client";

export const Route = createFileRoute("/admin/auth")({
  component: AdminAuthPage,
  head: () => ({ meta: [{ title: "Auth settings · Octanest" }] }),
});

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const FORBIDDEN = "You need admin access to manage auth settings.";

const PROVIDER_OPTIONS: { value: ProviderMode; label: string }[] = [
  { value: "local", label: "Local" },
  { value: "workos", label: "WorkOS" },
  { value: "oidc", label: "OIDC" },
];

const EMAIL_OPTIONS: { value: EmailProviderKind; label: string }[] = [
  { value: "log", label: "Log sink" },
  { value: "smtp", label: "SMTP" },
  { value: "resend", label: "Resend" },
];

type Phase =
  | { kind: "loading" }
  | { kind: "forbidden" }
  | { kind: "ready"; user: UserPublic; settings: AuthSettingsPublic }
  | { kind: "error"; message: string };

function EnvBadge({ show }: { show: boolean }) {
  if (!show) return null;
  return (
    <span className="inline-flex h-11 items-center rounded-md border border-border bg-muted/40 px-3 text-[14px] text-muted-foreground">
      Configured via ENV
    </span>
  );
}

function ModeSelect<T extends string>({
  id,
  label,
  value,
  options,
  onChange,
}: {
  id: string;
  label: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (next: T) => void;
}) {
  const current = options.find((o) => o.value === value) ?? options[0]!;
  return (
    <div className="flex flex-col gap-2">
      <Label htmlFor={id}>{label}</Label>
      <SelectRoot
        value={value}
        onValueChange={(next) => {
          if (typeof next === "string") onChange(next as T);
        }}
      >
        <SelectTrigger id={id} className="w-full min-w-[12rem]" aria-label={label}>
          <SelectValue>{current.label}</SelectValue>
          <SelectIcon>
            <ChevronDown aria-hidden size={16} />
          </SelectIcon>
        </SelectTrigger>
        <SelectPortal>
          <SelectPositioner sideOffset={4}>
            <SelectPopup>
              <SelectList>
                {options.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>
                    <SelectItemIndicator>
                      <Check aria-hidden size={16} className="text-primary" />
                    </SelectItemIndicator>
                    <SelectItemText>{opt.label}</SelectItemText>
                  </SelectItem>
                ))}
              </SelectList>
            </SelectPopup>
          </SelectPositioner>
        </SelectPortal>
      </SelectRoot>
    </div>
  );
}

function AdminAuthPage() {
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });
  const [providerMode, setProviderMode] = useState<ProviderMode>("local");
  const [emailProvider, setEmailProvider] = useState<EmailProviderKind>("log");
  const [fromAddress, setFromAddress] = useState("");
  const [workosClientId, setWorkosClientId] = useState("");
  const [oidcIssuer, setOidcIssuer] = useState("");
  const [oidcClientId, setOidcClientId] = useState("");
  const [flags, setFlags] = useState({
    smtp_configured: false,
    resend_configured: false,
    workos_api_key_configured: false,
    oidc_client_secret_configured: false,
  });
  const [formError, setFormError] = useState("");
  const [success, setSuccess] = useState("");
  const [pending, setPending] = useState(false);

  function applySettings(settings: AuthSettingsPublic) {
    setProviderMode(settings.provider_mode);
    setEmailProvider(settings.email_provider);
    setFromAddress(settings.from_address ?? "");
    setWorkosClientId(settings.workos_client_id ?? "");
    setOidcIssuer(settings.oidc_issuer ?? "");
    setOidcClientId(settings.oidc_client_id ?? "");
    setFlags({
      smtp_configured: settings.smtp_configured,
      resend_configured: settings.resend_configured,
      workos_api_key_configured: settings.workos_api_key_configured,
      oidc_client_secret_configured: settings.oidc_client_secret_configured,
    });
  }

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const me = await apiClient.auth.me();
        if (cancelled) return;
        if (!me.ok) {
          window.location.assign("/login?returnTo=/admin/auth");
          return;
        }
        if (!me.data.is_admin) {
          setPhase({ kind: "forbidden" });
          return;
        }
        const settingsRes = await apiClient.admin.auth.getSettings();
        if (cancelled) return;
        if (!settingsRes.ok) {
          if (settingsRes.error.code === "admin.forbidden") {
            setPhase({ kind: "forbidden" });
            return;
          }
          setPhase({
            kind: "error",
            message: settingsRes.error.message || NETWORK_ERROR,
          });
          return;
        }
        applySettings(settingsRes.data);
        setPhase({ kind: "ready", user: me.data, settings: settingsRes.data });
      } catch {
        if (cancelled) return;
        setPhase({ kind: "error", message: NETWORK_ERROR });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  async function saveSettings() {
    setFormError("");
    setSuccess("");
    setPending(true);
    try {
      const res = await apiClient.admin.auth.updateSettings({
        provider_mode: providerMode,
        email_provider: emailProvider,
        from_address: fromAddress.trim() || null,
        workos_client_id:
          providerMode === "workos" ? workosClientId.trim() || null : null,
        oidc_issuer: providerMode === "oidc" ? oidcIssuer.trim() || null : null,
        oidc_client_id:
          providerMode === "oidc" ? oidcClientId.trim() || null : null,
      });
      if (!res.ok) {
        setFormError(res.error.message || "Could not save auth settings.");
        setPending(false);
        return;
      }
      applySettings(res.data);
      setPhase((prev) =>
        prev.kind === "ready"
          ? { kind: "ready", user: prev.user, settings: res.data }
          : prev,
      );
      setSuccess("Auth settings saved.");
      setPending(false);
    } catch {
      setFormError(NETWORK_ERROR);
      setPending(false);
    }
  }

  if (phase.kind === "loading") {
    return (
      <div className="mx-auto max-w-2xl px-4 py-16">
        <p className="text-[16px] text-muted-foreground">Loading…</p>
      </div>
    );
  }

  if (phase.kind === "forbidden") {
    return (
      <div className="mx-auto max-w-2xl px-4 py-16">
        <p className="text-[16px] text-muted-foreground">{FORBIDDEN}</p>
      </div>
    );
  }

  if (phase.kind === "error") {
    return (
      <div className="mx-auto max-w-2xl px-4 py-16">
        <p className="text-[16px] text-destructive">{phase.message}</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl px-4 py-16">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
        Auth settings
      </h1>
      <p className="mt-2 text-[16px] text-muted-foreground">
        Instance provider mode and email delivery. Secrets stay in environment
        variables.
      </p>

      <div className="mt-8 flex flex-col gap-6">
        {formError ? (
          <p role="alert" className="text-[14px] text-destructive">
            {formError}
          </p>
        ) : null}
        {success ? (
          <p role="status" className="text-[14px] text-foreground">
            {success}
          </p>
        ) : null}

        <ModeSelect
          id="auth-provider"
          label="Auth provider"
          value={providerMode}
          options={PROVIDER_OPTIONS}
          onChange={setProviderMode}
        />

        {providerMode === "workos" ? (
          <div className="flex flex-col gap-4 rounded-md border border-border p-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="workos-client-id">WorkOS client ID</Label>
              <Input
                id="workos-client-id"
                value={workosClientId}
                onChange={(e) => setWorkosClientId(e.currentTarget.value)}
                autoComplete="off"
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label>WorkOS API key</Label>
              <EnvBadge show={flags.workos_api_key_configured} />
              {!flags.workos_api_key_configured ? (
                <p className="text-[14px] text-muted-foreground">
                  Not set — configure WORKOS_API_KEY in the environment.
                </p>
              ) : null}
            </div>
          </div>
        ) : null}

        {providerMode === "oidc" ? (
          <div className="flex flex-col gap-4 rounded-md border border-border p-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="oidc-issuer">OIDC issuer</Label>
              <Input
                id="oidc-issuer"
                value={oidcIssuer}
                onChange={(e) => setOidcIssuer(e.currentTarget.value)}
                autoComplete="off"
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="oidc-client-id">OIDC client ID</Label>
              <Input
                id="oidc-client-id"
                value={oidcClientId}
                onChange={(e) => setOidcClientId(e.currentTarget.value)}
                autoComplete="off"
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label>OIDC client secret</Label>
              <EnvBadge show={flags.oidc_client_secret_configured} />
              {!flags.oidc_client_secret_configured ? (
                <p className="text-[14px] text-muted-foreground">
                  Not set — configure OCTANEST_OIDC_CLIENT_SECRET in the
                  environment.
                </p>
              ) : null}
            </div>
          </div>
        ) : null}

        <ModeSelect
          id="email-delivery"
          label="Email delivery"
          value={emailProvider}
          options={EMAIL_OPTIONS}
          onChange={setEmailProvider}
        />

        <div className="flex flex-col gap-4 rounded-md border border-border p-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="from-address">From address</Label>
            <Input
              id="from-address"
              value={fromAddress}
              onChange={(e) => setFromAddress(e.currentTarget.value)}
              placeholder="Octanest <noreply@example.com>"
              autoComplete="off"
            />
          </div>
          {emailProvider === "smtp" ? (
            <div className="flex flex-col gap-2">
              <Label>SMTP credentials</Label>
              <EnvBadge show={flags.smtp_configured} />
              {!flags.smtp_configured ? (
                <p className="text-[14px] text-muted-foreground">
                  Not set — configure OCTANEST_SMTP_URL in the environment.
                </p>
              ) : null}
            </div>
          ) : null}
          {emailProvider === "resend" ? (
            <div className="flex flex-col gap-2">
              <Label>Resend API key</Label>
              <EnvBadge show={flags.resend_configured} />
              {!flags.resend_configured ? (
                <p className="text-[14px] text-muted-foreground">
                  Not set — configure OCTANEST_RESEND_API_KEY in the
                  environment.
                </p>
              ) : null}
            </div>
          ) : null}
        </div>

        <Button
          type="button"
          disabled={pending}
          onClick={() => void saveSettings()}
          className="w-full sm:w-auto"
        >
          {pending ? "Working…" : "Save auth settings"}
        </Button>
      </div>
    </div>
  );
}
