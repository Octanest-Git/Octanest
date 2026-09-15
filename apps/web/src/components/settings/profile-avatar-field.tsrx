import { useDropzone } from "@octanejs/dropzone";
import type { UserPublic } from "@octanest/api-client";
import { AvatarPreview } from "@/components/avatar-preview";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

const ACCEPT = {
  "image/jpeg": [".jpg", ".jpeg"],
  "image/png": [".png"],
  "image/webp": [".webp"],
} as const;

const MAX_SIZE = 2 * 1024 * 1024;

export type ProfileAvatarFieldProps = {
  user: Pick<UserPublic, "display_name" | "username" | "avatar_url">;
  src?: string | null;
  status?: string;
  disabled?: boolean;
  onFile: (file: File) => void | Promise<void>;
  onReject: () => void;
};

/**
 * Profile picture control — the squircle is the drop/click target (GitHub-style).
 */
export function ProfileAvatarField(props: ProfileAvatarFieldProps) @{
  const dropzone = useDropzone({
    accept: ACCEPT,
    maxFiles: 1,
    maxSize: MAX_SIZE,
    multiple: false,
    disabled: props.disabled,
    onDropAccepted(files) {
      const file = files[0];
      if (file) void props.onFile(file);
    },
    onDropRejected() {
      props.onReject();
    },
  });

  const busy = dropzone.isDragActive || dropzone.isProcessing;
  const rootProps = dropzone.getRootProps({
    className: cn(
      "group relative inline-flex size-24 shrink-0 cursor-pointer outline-none",
      "focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
      props.disabled && "cursor-not-allowed opacity-60",
    ),
  });

  <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:gap-5">
    <div {...rootProps}>
      <input
        id="profile-avatar"
        aria-label="Upload profile picture"
        {...dropzone.getInputProps()}
      />
      <AvatarPreview
        user={props.user}
        src={props.src}
        size={96}
        className={cn(
          "transition-[filter,opacity]",
          busy && "opacity-70",
        )}
      />
      <span
        aria-hidden
        className={cn(
          "octanest-squircle pointer-events-none absolute inset-0 flex items-center justify-center",
          "bg-foreground/70 text-[13px] font-medium text-background opacity-0 transition-opacity",
          "group-hover:opacity-100 group-focus-visible:opacity-100",
          busy && "opacity-100",
        )}
      >
        {(dropzone.isDragActive
          ? "Drop"
          : dropzone.isProcessing
            ? "…"
            : "Change") as string}
      </span>
    </div>

    <div className="min-w-0 flex-1">
      <Label htmlFor="profile-avatar">Profile picture</Label>
      <p className="mt-1 text-[14px] leading-[1.4] text-muted-foreground">
        Click or drop an image. JPEG, PNG, or WebP · max 2 MB.
      </p>
      @if (props.status) {
        <p className="mt-2 truncate text-[14px] text-muted-foreground" role="status">
          {props.status as string}
        </p>
      }
    </div>
  </div>
}
