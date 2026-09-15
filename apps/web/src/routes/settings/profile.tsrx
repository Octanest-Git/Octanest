import { createFileRoute, useLoaderData } from "@octanejs/tanstack-router";
import { useForm } from "@octanejs/tanstack-form";
import { useQueryClient } from "@octanejs/tanstack-query";
import type { UserPublic } from "@octanest/api-client";
import { useEffect, useState } from "octane";
import { ProfileAvatarField } from "@/components/settings/profile-avatar-field";
import { AccountSettingsLayout, SettingsShell } from "@/components/settings/settings-shell";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { apiClient } from "@/lib/api-client";
import { clearSessionQueries, setAuthMeCache } from "@/lib/session-queries";
import { syncSessionPresenceHint } from "@/lib/session-hint";
import { fetchUserGetProfile } from "@/lib/ssr-auth";
import { toastError, toastSuccess, toastWarning } from "@/lib/toast";
import { cn } from "@/lib/utils";

export type ProfileLoaderData =
  | { kind: "unauthenticated" }
  | { kind: "error"; message: string }
  | { kind: "ready"; user: UserPublic };

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const BIO_MAX = 160;

export const Route = createFileRoute("/settings/profile")({
  component: ProfilePage,
  head: () => ({ meta: [{ title: "Profile · Octanest" }] }),
  loader: async (): Promise<ProfileLoaderData> => {
    try {
      const res = await fetchUserGetProfile();
      if (!res.ok) {
        if (res.error.code === "auth.unauthenticated") {
          return { kind: "unauthenticated" };
        }
        return {
          kind: "error",
          message: res.error.message || "Could not load profile.",
        };
      }
      return { kind: "ready", user: res.data };
    } catch {
      return { kind: "error", message: NETWORK_ERROR };
    }
  },
});

type Phase =
  | { kind: "loading" }
  | { kind: "ready"; user: UserPublic }
  | { kind: "error"; message: string };

/** Exported for profile.integration.test.ts (RESEARCH P1 / D-QH-03). */
export function ProfilePage() @{
  const queryClient = useQueryClient();
  const loaderData = useLoaderData({ from: "/settings/profile" }) as ProfileLoaderData | undefined;

  useEffect(() => {
    if (loaderData?.kind === "unauthenticated") {
      window.location.assign("/login?returnTo=/settings/profile");
    }
  }, [loaderData]);

  const initialUser = loaderData?.kind === "ready" ? loaderData.user : null;

  const [phase, setPhase] = useState<Phase>(() => {
    if (loaderData?.kind === "ready") return { kind: "ready", user: loaderData.user };
    if (loaderData?.kind === "error") {
      return { kind: "error", message: loaderData.message };
    }
    if (loaderData?.kind === "unauthenticated") {
      return { kind: "loading" };
    }
    return { kind: "loading" };
  });
  const [avatarUrl, setAvatarUrl] = useState<string | null | undefined>(
    () => initialUser?.avatar_url,
  );
  const [avatarUploadPending, setAvatarUploadPending] = useState(false);
  const [avatarRemovePending, setAvatarRemovePending] = useState(false);
  const [pending, setPending] = useState(false);
  const [branchPending, setBranchPending] = useState(false);
  const [logoutPending, setLogoutPending] = useState(false);
  const [confirmLogoutAll, setConfirmLogoutAll] = useState(false);

  const profileForm = useForm({
    defaultValues: {
      display_name: initialUser?.display_name ?? "",
      username: initialUser?.username ?? "",
      bio: initialUser?.bio ?? "",
    },
    onSubmit: async ({ value }) => {
      setPending(true);
      const previousUsername = phase.kind === "ready" ? phase.user.username : "";
      try {
        const res = await apiClient.user.updateProfile({
          display_name: value.display_name.trim(),
          username: value.username.trim(),
          bio: value.bio.slice(0, BIO_MAX),
        });
        if (!res.ok) {
          toastError(res.error.message || "Could not save profile. Try again.");
          setPending(false);
          return;
        }
        setPhase({ kind: "ready", user: res.data });
        profileForm.setFieldValue("display_name", res.data.display_name ?? "");
        profileForm.setFieldValue("username", res.data.username ?? "");
        profileForm.setFieldValue("bio", res.data.bio ?? "");
        branchForm.setFieldValue("default_branch", res.data.default_branch?.trim() || "main");
        setAvatarUrl(res.data.avatar_url);
        setAuthMeCache(queryClient, res.data);
        if (previousUsername && res.data.username && previousUsername !== res.data.username) {
          toastSuccess(
            `Username updated to @${res.data.username}`,
            `Update git remotes to /${res.data.username}/{repo}.git.`,
          );
        } else {
          toastSuccess("Profile saved.");
        }
        setPending(false);
      } catch {
        toastError(NETWORK_ERROR);
        setPending(false);
      }
    },
  });

  const branchForm = useForm({
    defaultValues: {
      default_branch: initialUser?.default_branch?.trim() || "main",
    },
    onSubmit: async ({ value }) => {
      setBranchPending(true);
      const profileValues = profileForm.state.values;
      try {
        const res = await apiClient.user.updateProfile({
          display_name: profileValues.display_name.trim(),
          username: profileValues.username.trim(),
          bio: profileValues.bio.slice(0, BIO_MAX),
          default_branch: value.default_branch.trim() || "main",
        });
        if (!res.ok) {
          toastError(res.error.message || "Could not save default branch. Try again.");
          setBranchPending(false);
          return;
        }
        setPhase({ kind: "ready", user: res.data });
        branchForm.setFieldValue("default_branch", res.data.default_branch?.trim() || "main");
        setAuthMeCache(queryClient, res.data);
        toastSuccess("Default branch saved.");
        setBranchPending(false);
      } catch {
        toastError(NETWORK_ERROR);
        setBranchPending(false);
      }
    },
  });

  useEffect(() => {
    if (!loaderData) return;
    if (loaderData.kind === "ready") {
      const user = loaderData.user;
      profileForm.setFieldValue("display_name", user.display_name ?? "");
      profileForm.setFieldValue("username", user.username ?? "");
      profileForm.setFieldValue("bio", user.bio ?? "");
      branchForm.setFieldValue("default_branch", user.default_branch?.trim() || "main");
      setAvatarUrl(user.avatar_url);
      setPhase({ kind: "ready", user });
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
        const res = await apiClient.user.getProfile();
        if (cancelled) return;
        if (!res.ok) {
          window.location.assign("/login?returnTo=/settings/profile");
          return;
        }
        const user = res.data;
        profileForm.setFieldValue("display_name", user.display_name ?? "");
        profileForm.setFieldValue("username", user.username ?? "");
        profileForm.setFieldValue("bio", user.bio ?? "");
        branchForm.setFieldValue("default_branch", user.default_branch?.trim() || "main");
        setAvatarUrl(user.avatar_url);
        setPhase({ kind: "ready", user });
      } catch {
        if (cancelled) return;
        setPhase({ kind: "error", message: NETWORK_ERROR });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [loaderData]);

  async function onAvatarSelected(file: File | undefined) {
    if (!file) return;
    setAvatarUploadPending(true);
    const formData = new FormData();
    formData.append("avatar", file);
    try {
      const res = await fetch("/api/user/avatar", {
        method: "POST",
        credentials: "include",
        body: formData,
      });
      const json = (await res.json()) as {
        ok?: boolean;
        avatar_url?: string;
        error?: { message?: string };
      };
      if (!res.ok || !json.ok || !json.avatar_url) {
        toastError(json.error?.message || "Avatar upload failed. Try again.");
        setAvatarUploadPending(false);
        throw new Error("avatar upload failed");
      }
      const nextUrl = `${json.avatar_url}?t=${Date.now()}`;
      setAvatarUrl(nextUrl);
      setPhase((prev) => {
        if (prev.kind !== "ready") return prev;
        const nextUser = { ...prev.user, avatar_url: json.avatar_url };
        setAuthMeCache(queryClient, nextUser);
        return { kind: "ready", user: nextUser };
      });
      toastSuccess("Profile picture updated.");
      setAvatarUploadPending(false);
    } catch (err) {
      setAvatarUploadPending(false);
      if (err instanceof Error && err.message === "avatar upload failed") {
        throw err;
      }
      toastError(NETWORK_ERROR);
      throw err instanceof Error ? err : new Error(NETWORK_ERROR);
    }
  }

  async function onAvatarRemove() {
    setAvatarRemovePending(true);
    try {
      const res = await fetch("/api/user/avatar", {
        method: "DELETE",
        credentials: "include",
      });
      const json = (await res.json()) as {
        ok?: boolean;
        error?: { message?: string };
      };
      if (!res.ok || !json.ok) {
        toastError(json.error?.message || "Could not remove profile picture.");
        setAvatarRemovePending(false);
        return;
      }
      setAvatarUrl(null);
      setPhase((prev) => {
        if (prev.kind !== "ready") return prev;
        const nextUser = { ...prev.user, avatar_url: null };
        setAuthMeCache(queryClient, nextUser);
        return { kind: "ready", user: nextUser };
      });
      toastSuccess("Profile picture removed.");
      setAvatarRemovePending(false);
    } catch {
      toastError(NETWORK_ERROR);
      setAvatarRemovePending(false);
    }
  }

  async function logoutThisDevice() {
    setLogoutPending(true);
    try {
      await apiClient.auth.logout();
    } catch {
      /* still leave */
    }
    syncSessionPresenceHint(false);
    clearSessionQueries(queryClient);
    window.location.assign("/");
  }

  async function logoutAllDevices() {
    setLogoutPending(true);
    try {
      await apiClient.auth.logoutAll();
    } catch {
      /* still leave */
    }
    syncSessionPresenceHint(false);
    clearSessionQueries(queryClient);
    window.location.assign("/");
  }

  @if (phase.kind === "loading") {
    <></>;
  } @else {
    @if (phase.kind === "error") {
      <SettingsShell bare>
        <p className="text-[16px] text-destructive">{phase.message as string}</p>
      </SettingsShell>;
    } @else {
      <AccountSettingsLayout active="profile" data-testid="settings-profile-page">
        <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
          Profile
        </h1>

        <div className="mt-8 flex flex-col gap-6">
          <ProfileAvatarField
            user={phase.user}
            src={avatarUrl}
            uploadPending={avatarUploadPending}
            removePending={avatarRemovePending}
            onCroppedFile={(file) => onAvatarSelected(file)}
            onRemove={() => onAvatarRemove()}
            onReject={() => {
              toastWarning("Choose a JPEG, PNG, or WebP image under 2 MB.");
            }}
          />

          <form
            method="post"
            action="#"
            className="flex flex-col gap-4"
            onSubmit={(e) => {
              e.preventDefault();
              void profileForm.handleSubmit();
            }}
          >
            <profileForm.Field name="display_name">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor="display-name">Display name</Label>
                  <Input
                    id="display-name"
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onInput={(e) => field.handleChange((e.currentTarget as HTMLInputElement).value)}
                    autoComplete="name"
                  />
                </div>
              )}
            </profileForm.Field>
            <profileForm.Field name="username">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor="username">Username</Label>
                  <Input
                    id="username"
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onInput={(e) => field.handleChange((e.currentTarget as HTMLInputElement).value)}
                    autoComplete="username"
                  />
                  <p className="text-[14px] text-muted-foreground">
                    1–39 characters; letters, numbers, and hyphens. Can't start or end with a
                    hyphen.
                  </p>
                </div>
              )}
            </profileForm.Field>
            <profileForm.Field name="bio">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor="bio">Bio</Label>
                  <Textarea
                    id="bio"
                    rows={4}
                    maxLength={BIO_MAX}
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onInput={(e) =>
                      field.handleChange(
                        (e.currentTarget as HTMLTextAreaElement).value.slice(0, BIO_MAX),
                      )
                    }
                  />
                  <p className="text-[14px] text-muted-foreground">
                    Optional · up to 160 characters
                  </p>
                </div>
              )}
            </profileForm.Field>
            <Button type="submit" disabled={pending} className="w-full sm:w-auto">
              {(pending ? "Working…" : "Save profile") as string}
            </Button>
          </form>

          <hr className="border-border" />

          <form
            method="post"
            action="#"
            className="flex flex-col gap-4"
            onSubmit={(e) => {
              e.preventDefault();
              void branchForm.handleSubmit();
            }}
          >
            <branchForm.Field name="default_branch">
              {(field) => (
                <div className="flex flex-col gap-2">
                  <Label htmlFor="default-branch">Default branch name</Label>
                  <Input
                    id="default-branch"
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onInput={(e) => field.handleChange((e.currentTarget as HTMLInputElement).value)}
                    autoComplete="off"
                  />
                  <p className="text-[14px] text-muted-foreground">
                    Applies to new repositories you create. Existing repos keep their current
                    default branch.
                  </p>
                </div>
              )}
            </branchForm.Field>
            <Button type="submit" disabled={branchPending} className="w-full sm:w-auto">
              {(branchPending ? "Working…" : "Save default branch") as string}
            </Button>
          </form>

          <hr className="border-border" />

          <div className="flex flex-wrap gap-3">
            <Button
              type="button"
              variant="ghost"
              disabled={logoutPending}
              onClick={() => void logoutThisDevice()}
            >
              Log out
            </Button>
            <Button
              type="button"
              variant="ghost"
              disabled={logoutPending}
              className={cn("border border-destructive text-destructive hover:bg-destructive/10")}
              onClick={() => setConfirmLogoutAll(true)}
            >
              Log out all devices
            </Button>
          </div>
        </div>

        @if (confirmLogoutAll) {
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="logout-all-title"
            className="fixed inset-0 z-50 flex items-center justify-center bg-foreground/40 p-4"
          >
            <div className="w-full max-w-md rounded-md border border-border bg-card p-6 shadow-lg">
              <h2
                id="logout-all-title"
                className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground"
              >
                Log out all devices
              </h2>
              <p className="mt-3 text-[16px] text-muted-foreground">
                Sign out everywhere? You'll need to sign in again on each device.
              </p>
              <div className="mt-6 flex flex-wrap gap-3">
                <Button
                  type="button"
                  disabled={logoutPending}
                  className="bg-destructive text-white hover:brightness-110"
                  onClick={() => void logoutAllDevices()}
                >
                  Log out all devices
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  disabled={logoutPending}
                  onClick={() => setConfirmLogoutAll(false)}
                >
                  Stay signed in
                </Button>
              </div>
            </div>
          </div>;
        }
      </AccountSettingsLayout>;
    }
  }
}
