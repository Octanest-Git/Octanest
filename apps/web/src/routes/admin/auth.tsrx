import { Check, ChevronDown } from "@octanejs/lucide";
import { createFileRoute } from "@octanejs/tanstack-router";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import type {
  AuthSettingsPublic,
  EmailProviderKind,
  ProviderMode,
} from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { apiClient } from "@/lib/api-client";
import {
  adminAuthSettingsQueryOptions,
  authSessionQueryOptions,
  clearSessionQueries,
  setAdminAuthSettingsCache,
} from "@/lib/session-queries";
import { syncSessionPresenceHint } from "@/lib/session-hint";
import { AdminAuthSkeleton } from "@/components/page-skeletons";
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
import { Switch } from "@/components/ui/switch";

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

function EnvBadge({ show }: { show: boolean }) @{
  @if (show) {
    <span className="inline-flex h-11 items-center rounded-md border border-border bg-muted/40 px-3 text-[14px] text-muted-foreground">
      Configured via ENV
    </span>
  }
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
}) @{
  const current = options.find((o) => o.value === value) ?? options[0]!;

  <div className="flex flex-col gap-2">
    <Label htmlFor={id}>{label as string}</Label>
    <SelectRoot
      value={value}
      onValueChange={(next) => {
        if (typeof next === "string") onChange(next as T);
      }}
    >
      <SelectTrigger id={id} className="w-full min-w-[12rem]" aria-label={label}>
        <SelectValue>{current.label as string}</SelectValue>
        <SelectIcon>
          <ChevronDown aria-hidden size={16} />
        </SelectIcon>
      </SelectTrigger>
      <SelectPortal>
        <SelectPositioner sideOffset={4}>
          <SelectPopup>
            <SelectList>
              @for (const opt of options; key opt.value) {
                <SelectItem value={opt.value}>
                  <SelectItemIndicator>
                    <Check aria-hidden size={16} className="text-primary" />
                  </SelectItemIndicator>
                  <SelectItemText>{opt.label as string}</SelectItemText>
                </SelectItem>
              }
            </SelectList>
          </SelectPopup>
        </SelectPositioner>
      </SelectPortal>
    </SelectRoot>
  </div>
}

/** Exported for D-08 integration tests (Allow open signup Switch). */
export function AdminAuthPage() @{
  const queryClient = useQueryClient();
  const me = useQuery(authSessionQueryOptions());
  const isSysAdmin = me.isSuccess && me.data?.role === "sys-admin";
  const settings = useQuery({
    ...adminAuthSettingsQueryOptions(),
    enabled: isSysAdmin,
  });

  const [providerMode, setProviderMode] = useState<ProviderMode>("local");
  const [allowSignup, setAllowSignup] = useState(false);
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
  const [draftReady, setDraftReady] = useState(false);

  function applySettings(next: AuthSettingsPublic) {
    setProviderMode(next.provider_mode);
    setAllowSignup(next.allow_signup === true);
    setEmailProvider(next.email_provider);
    setFromAddress(next.from_address ?? "");
    setWorkosClientId(next.workos_client_id ?? "");
    setOidcIssuer(next.oidc_issuer ?? "");
    setOidcClientId(next.oidc_client_id ?? "");
    setFlags({
      smtp_configured: next.smtp_configured,
      resend_configured: next.resend_configured,
      workos_api_key_configured: next.workos_api_key_configured,
      oidc_client_secret_configured: next.oidc_client_secret_configured,
    });
    setDraftReady(true);
  }

  useEffect(() => {
    if (me.isPending) return;
    if (me.isError) return;
    if (me.data === null) {
      window.location.assign("/login?returnTo=/admin/auth");
    }
  }, [me.isPending, me.isError, me.data]);

  useEffect(() => {
    if (settings.isSuccess && settings.data) {
      applySettings(settings.data);
    }
  }, [settings.isSuccess, settings.dataUpdatedAt]);

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
        allow_signup: allowSignup,
      });
      if (!res.ok) {
        setFormError(res.error.message || "Could not save auth settings.");
        setPending(false);
        return;
      }
      applySettings(res.data);
      setAdminAuthSettingsCache(queryClient, res.data);
      setSuccess("Auth settings saved.");
      setPending(false);
    } catch {
      setFormError(NETWORK_ERROR);
      setPending(false);
    }
  }

  const settingsErrorCode =
    settings.isError && settings.error && typeof settings.error === "object"
      ? (settings.error as { code?: string }).code
      : undefined;

  const loading =
    me.isPending ||
    (isSysAdmin && settings.isPending) ||
    (isSysAdmin && settings.isSuccess && !draftReady);

  const forbidden =
    (!me.isPending &&
      me.isSuccess &&
      me.data !== null &&
      me.data.role !== "sys-admin") ||
    settingsErrorCode === "admin.forbidden";

  const loadError =
    me.isError ||
    (settings.isError && settingsErrorCode !== "admin.forbidden");

  @if (loading) {
    <AdminAuthSkeleton />
  } @else {
    @if (forbidden) {
      <div className="mx-auto max-w-2xl px-4 py-16">
        <p className="text-[16px] text-muted-foreground">{FORBIDDEN}</p>
      </div>
    } @else {
      @if (loadError) {
        <div className="mx-auto max-w-2xl px-4 py-16">
          <p className="text-[16px] text-destructive">{NETWORK_ERROR}</p>
        </div>
      } @else {
    <div className="mx-auto max-w-2xl px-4 py-16">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
        Auth settings
      </h1>
      <p className="mt-2 text-[16px] text-muted-foreground">
        Instance provider mode and email delivery. Secrets stay in environment
        variables.
      </p>

      <div className="mt-8 flex flex-col gap-6">
        @if (formError) {
          <p role="alert" className="text-[14px] text-destructive">
            {formError as string}
          </p>
        }
        @if (success) {
          <p role="status" className="text-[14px] text-foreground">
            {success as string}
          </p>
        }

        <ModeSelect
          id="auth-provider"
          label="Auth provider"
          value={providerMode}
          options={PROVIDER_OPTIONS}
          onChange={setProviderMode}
        />

        <div className="flex flex-col gap-2">
          <div className="flex min-h-11 items-center gap-2">
            <Switch
              id="admin-allow-signup"
              checked={allowSignup}
              onCheckedChange={(checked) => setAllowSignup(checked === true)}
            />
            <Label htmlFor="admin-allow-signup">Allow open signup</Label>
          </div>
          <p className="text-[14px] font-normal leading-[1.4] text-muted-foreground">
            When off, new accounts can’t self-register. You can change this later in
            Auth settings.
          </p>
        </div>

        @if (providerMode === "workos") {
          <div className="flex flex-col gap-4 rounded-md border border-border p-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="workos-client-id">WorkOS client ID</Label>
              <Input
                id="workos-client-id"
                value={workosClientId}
                onInput={(e) =>
                  setWorkosClientId((e.currentTarget as HTMLInputElement).value)
                }
                autoComplete="off"
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label>WorkOS API key</Label>
              <EnvBadge show={flags.workos_api_key_configured} />
              @if (!flags.workos_api_key_configured) {
                <p className="text-[14px] text-muted-foreground">
                  Not set — configure WORKOS_API_KEY in the environment.
                </p>
              }
            </div>
          </div>
        }

        @if (providerMode === "oidc") {
          <div className="flex flex-col gap-4 rounded-md border border-border p-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="oidc-issuer">OIDC issuer</Label>
              <Input
                id="oidc-issuer"
                value={oidcIssuer}
                onInput={(e) =>
                  setOidcIssuer((e.currentTarget as HTMLInputElement).value)
                }
                autoComplete="off"
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="oidc-client-id">OIDC client ID</Label>
              <Input
                id="oidc-client-id"
                value={oidcClientId}
                onInput={(e) =>
                  setOidcClientId((e.currentTarget as HTMLInputElement).value)
                }
                autoComplete="off"
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label>OIDC client secret</Label>
              <EnvBadge show={flags.oidc_client_secret_configured} />
              @if (!flags.oidc_client_secret_configured) {
                <p className="text-[14px] text-muted-foreground">
                  Not set — configure OCTANEST_OIDC_CLIENT_SECRET in the
                  environment.
                </p>
              }
            </div>
          </div>
        }

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
              onInput={(e) =>
                setFromAddress((e.currentTarget as HTMLInputElement).value)
              }
              placeholder="Octanest <noreply@example.com>"
              autoComplete="off"
            />
          </div>
          @if (emailProvider === "smtp") {
            <div className="flex flex-col gap-2">
              <Label>SMTP credentials</Label>
              <EnvBadge show={flags.smtp_configured} />
              @if (!flags.smtp_configured) {
                <p className="text-[14px] text-muted-foreground">
                  Not set — configure OCTANEST_SMTP_URL in the environment.
                </p>
              }
            </div>
          }
          @if (emailProvider === "resend") {
            <div className="flex flex-col gap-2">
              <Label>Resend API key</Label>
              <EnvBadge show={flags.resend_configured} />
              @if (!flags.resend_configured) {
                <p className="text-[14px] text-muted-foreground">
                  Not set — configure OCTANEST_RESEND_API_KEY in the
                  environment.
                </p>
              }
            </div>
          }
        </div>

        <Button
          type="button"
          disabled={pending}
          onClick={() => void saveSettings()}
          className="w-full sm:w-auto"
        >
          {(pending ? "Working…" : "Save auth settings") as string}
        </Button>

        <FactoryResetSection />
      </div>
    </div>
      }
    }
  }
}

function FactoryResetSection() @{
  const queryClient = useQueryClient();
  const [phrase, setPhrase] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");

  async function runReset() {
    setError("");
    setPending(true);
    try {
      const res = await apiClient.admin.auth.factoryReset({
        confirmation: phrase.trim(),
      });
      if (!res.ok) {
        setError(res.error.message || "Factory reset failed.");
        setPending(false);
        return;
      }
      syncSessionPresenceHint(false);
      clearSessionQueries(queryClient);
      window.location.assign("/setup");
    } catch {
      setError(NETWORK_ERROR);
      setPending(false);
    }
  }

  <div className="mt-12 flex flex-col gap-4 rounded-md border border-destructive/40 p-4">
    <h2 className="font-[family-name:var(--font-display)] text-[18px] font-semibold text-destructive">
      Danger zone
    </h2>
    <p className="text-[14px] leading-[1.4] text-muted-foreground">
      Wipe all users, sessions, and auth identities, then return this instance to
      the empty setup wizard. This cannot be undone. Type{" "}
      <span className="font-medium text-foreground">RESET</span> to confirm.
    </p>
    @if (error) {
      <p role="alert" className="text-[14px] text-destructive">
        {error as string}
      </p>
    }
    <div className="flex flex-col gap-2">
      <Label htmlFor="factory-reset-confirm">Confirmation</Label>
      <Input
        id="factory-reset-confirm"
        value={phrase}
        onInput={(e) => setPhrase((e.currentTarget as HTMLInputElement).value)}
        autoComplete="off"
        placeholder="RESET"
      />
    </div>
    <Button
      type="button"
      variant="destructive"
      disabled={pending || phrase.trim() !== "RESET"}
      onClick={() => void runReset()}
      className="w-full sm:w-auto"
    >
      {(pending ? "Resetting…" : "Reset instance database") as string}
    </Button>
  </div>
}
