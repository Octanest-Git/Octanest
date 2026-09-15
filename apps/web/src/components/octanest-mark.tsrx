import { cn } from "@/lib/utils";

export function OctanestMark({
  size = 32,
  className,
}: {
  size?: number;
  className?: string;
}) @{
  <img
    src="/octanest-mark.png"
    alt="Octanest"
    width={size}
    height={size}
    className={cn("octanest-squircle block shrink-0", className)}
    style={{ width: `${size}px`, height: `${size}px` }}
  />;
}
