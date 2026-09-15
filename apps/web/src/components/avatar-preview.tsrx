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

/**
 * Squircle avatar preview — stable outer box so empty/filled states share size
 * (avoids layout shift + img↔span root swaps that break Octane DOM updates).
 */
export function AvatarPreview(props: AvatarPreviewProps) @{
  const size = props.size ?? 64;
  const url = props.src ?? props.user.avatar_url;
  const dim = `${size}px`;

  <div
    className={cn(
      "octanest-squircle relative block shrink-0 overflow-hidden bg-muted",
      props.className,
    )}
    style={{ width: dim, height: dim, minWidth: dim, minHeight: dim }}
  >
    @if (url) {
      <img
        src={url as string}
        alt=""
        width={size}
        height={size}
        className="block size-full object-cover"
      />
    } @else {
      <span
        aria-hidden
        className="flex size-full items-center justify-center text-[18px] font-semibold leading-none text-foreground"
      >
        {initialsFor(props.user) as string}
      </span>
    }
  </div>
}
