import type { UserPublic } from "@octanest/api-client";
import { cn } from "@/lib/utils";

function initialsFor(user: Pick<UserPublic, "display_name" | "username">): string {
  const source = user.display_name?.trim() || user.username || "?";
  const parts = source.split(/\s+/).filter(Boolean);
  if (parts.length >= 2) {
    return `${parts[0]![0] ?? ""}${parts[1]![0] ?? ""}`.toUpperCase();
  }
  return source.slice(0, 2).toUpperCase();
}

type AvatarPreviewProps = {
  user: Pick<UserPublic, "display_name" | "username" | "avatar_url">;
  /** Preview URL override (e.g. after upload before reload). */
  src?: string | null;
  size?: number;
  className?: string;
};

/** 64×64 squircle avatar preview — reuses `octanest-squircle` clip language. */
export function AvatarPreview({
  user,
  src,
  size = 64,
  className,
}: AvatarPreviewProps) {
  const url = src ?? user.avatar_url;
  const dim = `${size}px`;

  if (url) {
    return (
      <img
        src={url}
        alt=""
        width={size}
        height={size}
        className={cn("octanest-squircle block shrink-0 object-cover", className)}
        style={{ width: dim, height: dim }}
      />
    );
  }

  return (
    <span
      aria-hidden
      className={cn(
        "octanest-squircle inline-flex shrink-0 items-center justify-center bg-muted text-[18px] font-semibold text-foreground",
        className,
      )}
      style={{ width: dim, height: dim }}
    >
      {initialsFor(user)}
    </span>
  );
}
