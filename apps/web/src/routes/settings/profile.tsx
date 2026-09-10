import { createFileRoute } from "@octanejs/tanstack-router";
import type { UserPublic } from "@octanest/api-client";
import { useEffect, useRef, useState } from "octane";
import { AvatarPreview } from "@/components/avatar-preview";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { apiClient } from "@/lib/api-client";
import { cn } from "@/lib/utils";

export const Route = createFileRoute("/settings/profile")({
  component: ProfilePage,
  head: () => ({ meta: [{ title: "Profile · Octanest" }] }),
});

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";
const BIO_MAX = 160;

type Phase =
  | { kind: "loading" }
  | { kind: "ready"; user: UserPublic }
  | { kind: "error"; message: string };

function ProfilePage() {
  const [phase, setPhase] = useState<Phase>({ kind: "loading" });
  const [displayName, setDisplayName] = useState("");
  const [username, setUsername] = useState("");
  const [bio, setBio] = useState("");
  const [avatarUrl, setAvatarUrl] = useState<string | null | undefined>(undefined);
  const [avatarStatus, setAvatarStatus] = useState("");
  const [formError, setFormError] = useState("");
  const [success, setSuccess] = useState("");
  const [pending, setPending] = useState(false);
  const [logoutPending, setLogoutPending] = useState(false);
  const [confirmLogoutAll, setConfirmLogoutAll] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await apiClient.user.getProfile();
        if (cancelled) return;
        if (!res.ok) {
          window.location.assign("/login?returnTo=/settings/profile");
          return;
        }
        const user = res.data;
        setDisplayName(user.display_name ?? "");
        setUsername(user.username ?? "");
        setBio(user.bio ?? "");
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
  }, []);

  async function saveProfile() {
    setFormError("");
    setSuccess("");
    setPending(true);
    try {
      const res = await apiClient.user.updateProfile({
        display_name: displayName.trim(),
        username: username.trim(),
        bio: bio.slice(0, BIO_MAX),
      });
      if (!res.ok) {
        setFormError(res.error.message || "Could not save profile. Try again.");
        setPending(false);
        return;
      }
      setPhase({ kind: "ready", user: res.data });
      setDisplayName(res.data.display_name ?? "");
      setUsername(res.data.username ?? "");
      setBio(res.data.bio ?? "");
      setAvatarUrl(res.data.avatar_url);
      setSuccess("Profile saved.");
      setPending(false);
    } catch {
      setFormError(NETWORK_ERROR);
      setPending(false);
    }
  }

  async function onAvatarSelected(file: File | undefined) {
    if (!file) return;
    setAvatarStatus(file.name);
    setFormError("");
    setSuccess("");
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
        setFormError(json.error?.message || "Avatar upload failed. Try again.");
        setAvatarStatus("");
        return;
      }
      const nextUrl = `${json.avatar_url}?t=${Date.now()}`;
      setAvatarUrl(nextUrl);
      setAvatarStatus("Avatar updated.");
      setPhase((prev) =>
        prev.kind === "ready"
          ? { kind: "ready", user: { ...prev.user, avatar_url: json.avatar_url } }
          : prev,
      );
    } catch {
      setFormError(NETWORK_ERROR);
      setAvatarStatus("");
    }
  }

  async function logoutThisDevice() {
    setLogoutPending(true);
    try {
      await apiClient.auth.logout();
    } catch {
      /* still leave */
    }
    window.location.assign("/");
  }

  async function logoutAllDevices() {
    setLogoutPending(true);
    try {
      await apiClient.auth.logoutAll();
    } catch {
      /* still leave */
    }
    window.location.assign("/");
  }

  if (phase.kind === "loading") {
    return (
      <div className="mx-auto max-w-xl px-4 py-16">
        <p className="text-[16px] text-muted-foreground">Loading…</p>
      </div>
    );
  }

  if (phase.kind === "error") {
    return (
      <div className="mx-auto max-w-xl px-4 py-16">
        <p className="text-[16px] text-destructive">{phase.message}</p>
      </div>
    );
  }

  const user = phase.user;

  return (
    <div className="mx-auto max-w-xl px-4 py-16">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
        Profile
      </h1>

      <div className="mt-8 flex flex-col gap-6">
        <div className="flex items-center gap-4">
          <AvatarPreview user={user} src={avatarUrl} size={64} />
          <div className="flex min-w-0 flex-col gap-2">
            <input
              ref={fileInputRef}
              type="file"
              accept="image/jpeg,image/png,image/webp"
              className="sr-only"
              onChange={(e) => {
                const input = e.currentTarget;
                void onAvatarSelected(input.files?.[0]);
                input.value = "";
              }}
            />
            <Button
              type="button"
              variant="ghost"
              className="w-fit"
              onClick={() => fileInputRef.current?.click()}
            >
              Change avatar
            </Button>
            <p className="text-[14px] text-muted-foreground">
              JPEG, PNG, or WebP · max 2 MB
            </p>
            {avatarStatus ? (
              <p className="truncate text-[14px] text-muted-foreground">{avatarStatus}</p>
            ) : null}
          </div>
        </div>

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

        <div className="flex flex-col gap-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="display-name">Display name</Label>
            <Input
              id="display-name"
              value={displayName}
              onChange={(e) => setDisplayName(e.currentTarget.value)}
              autoComplete="name"
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="username">Username</Label>
            <Input
              id="username"
              value={username}
              onChange={(e) => setUsername(e.currentTarget.value)}
              autoComplete="username"
            />
            <p className="text-[14px] text-muted-foreground">
              1–39 characters; letters, numbers, and hyphens. Can’t start or end with
              a hyphen.
            </p>
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="bio">Bio</Label>
            <Textarea
              id="bio"
              rows={4}
              maxLength={BIO_MAX}
              value={bio}
              onChange={(e) => setBio(e.currentTarget.value.slice(0, BIO_MAX))}
            />
            <p className="text-[14px] text-muted-foreground">
              Optional · up to 160 characters
            </p>
          </div>
          <Button
            type="button"
            disabled={pending}
            onClick={() => void saveProfile()}
            className="w-full sm:w-auto"
          >
            {pending ? "Working…" : "Save profile"}
          </Button>
        </div>

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
            className={cn(
              "border border-destructive text-destructive hover:bg-destructive/10",
            )}
            onClick={() => setConfirmLogoutAll(true)}
          >
            Log out all devices
          </Button>
        </div>
      </div>

      {confirmLogoutAll ? (
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
              Sign out everywhere? You’ll need to sign in again on each device.
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
        </div>
      ) : null}
    </div>
  );
}
