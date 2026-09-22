import { createFileRoute, useLoaderData } from "@octanejs/tanstack-router";
import { sshKeyListQueryOptions, type SshKeyListItem, type UserPublic } from "@octanest/api-client";
import { useQuery } from "@octanejs/tanstack-query";
import { useEffect, useState } from "octane";
import { SshKeyAddForm } from "@/components/settings/ssh-key-add-form";
import { SshKeyList } from "@/components/settings/ssh-key-list";
import { AccountSettingsLayout, SettingsShell } from "@/components/settings/settings-shell";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";
import { fetchSessionMe, fetchSshKeyList } from "@/lib/ssr-auth";

export type SshKeysLoaderData =
  | { kind: "unauthenticated" }
  | { kind: "error"; message: string }
  | { kind: "ready"; user: UserPublic; keys: SshKeyListItem[] };

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

export const Route = createFileRoute("/settings/ssh-keys")({
  component: SshKeysPage,
  head: () => ({ meta: [{ title: "SSH keys · Octanest" }] }),
  loader: async (): Promise<SshKeysLoaderData> => {
    try {
      const res = await fetchSessionMe();
      if (!res.ok) {
        if (res.error.code === "auth.unauthenticated") {
          return { kind: "unauthenticated" };
        }
        return {
          kind: "error",
          message: res.error.message || "Could not load session.",
        };
      }
      const listRes = await fetchSshKeyList();
      if (!listRes.ok) {
        return {
          kind: "error",
          message: listRes.error.message || "Could not load SSH keys.",
        };
      }
      return { kind: "ready", user: res.data, keys: listRes.data };
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

type Phase =
  | { kind: "loading" }
  | { kind: "ready"; user: UserPublic }
  | { kind: "error"; message: string };

/** Exported for ssh-keys.integration.test.ts (GIT-04 / D-SSH-06). */
export function SshKeysPage() @{
  const loaderData = useLoaderData({ from: "/settings/ssh-keys" }) as SshKeysLoaderData | undefined;

  useEffect(() => {
    if (loaderData?.kind === "unauthenticated") {
      window.location.assign("/login?returnTo=/settings/ssh-keys");
    }
  }, [loaderData]);

  const [phase, setPhase] = useState<Phase>(() => {
    if (loaderData?.kind === "ready") {
      return { kind: "ready", user: loaderData.user };
    }
    if (loaderData?.kind === "error") {
      return { kind: "error", message: loaderData.message };
    }
    if (loaderData?.kind === "unauthenticated") {
      return { kind: "loading" };
    }
    return { kind: "loading" };
  });

  const [keys, setKeys] = useState<SshKeyListItem[] | undefined>(() =>
    loaderData?.kind === "ready" ? loaderData.keys : undefined,
  );
  const [showForm, setShowForm] = useState(false);

  useEffect(() => {
    if (!loaderData) return;
    if (loaderData.kind === "ready") {
      setPhase({ kind: "ready", user: loaderData.user });
      setKeys(loaderData.keys);
      return;
    }
    if (loaderData.kind === "error") {
      setPhase({ kind: "error", message: loaderData.message });
    }
  }, [loaderData]);

  useEffect(() => {
    if (loaderData) return;
    let cancelled = false;
    void (async () => {
      try {
        const res = await apiClient.auth.me();
        if (cancelled) return;
        if (!res.ok) {
          window.location.assign("/login?returnTo=/settings/ssh-keys");
          return;
        }
        const listRes = await apiClient.sshKey.list();
        if (cancelled) return;
        if (!listRes.ok) {
          setPhase({
            kind: "error",
            message: listRes.error.message || "Could not load SSH keys.",
          });
          return;
        }
        setKeys(listRes.data);
        setPhase({ kind: "ready", user: res.data });
      } catch {
        if (cancelled) return;
        setPhase({ kind: "error", message: NETWORK_ERROR });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [loaderData]);

  const verified = phase.kind === "ready" ? phase.user.email_verified === true : false;

  const keyList = useQuery({
    ...sshKeyListQueryOptions(apiClient),
    enabled: phase.kind === "ready",
    ...(keys !== undefined ? { initialData: keys } : {}),
  });
  const keyCount = keyList.data?.length ?? keys?.length ?? 0;
  const atCap = keyCount >= 25;
  const canAdd = verified && !atCap;

  @if (phase.kind === "loading") {
    <></>;
  } @else {
    @if (phase.kind === "error") {
      <SettingsShell bare>
        <p role="alert" className="text-[16px] text-destructive">
          {phase.message as string}
        </p>
      </SettingsShell>;
    } @else {
      <AccountSettingsLayout active="ssh-keys" data-testid="settings-ssh-keys-page">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="min-w-0">
            <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
              SSH keys
            </h1>
            <p className="mt-2 text-[16px] text-muted-foreground">
              Keys authenticate git over SSH as user git, and the same keys can sign commits when
              your local git uses <code className="text-[14px]">gpg.format=ssh</code>. Identity
              comes from the key fingerprint, not the SSH username.
            </p>
          </div>
          <Button type="button" disabled={!canAdd} onClick={() => setShowForm(true)}>
            Add SSH key
          </Button>
        </div>
        @if (!verified) {
          <p className="mt-3 text-[14px] text-muted-foreground">
            Verify your email to add an SSH key.
          </p>;
        } @else {
          @if (atCap) {
            <p className="mt-3 text-[14px] text-muted-foreground">Maximum of 25 SSH keys</p>;
          }
        }
        @if (showForm && canAdd) {
          <SshKeyAddForm disabled={!canAdd} onAdded={() => setShowForm(false)} />;
        }
        <SshKeyList enabled={true} initialData={keys} />
      </AccountSettingsLayout>;
    }
  }
}
