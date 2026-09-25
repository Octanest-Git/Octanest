import type { LabelPublic } from "@oxidean/api-client";
import { contrastRatio } from "@/lib/contrast";
import { cn } from "@/lib/utils";

type ChipStyle = { backgroundColor: string; color: string; borderColor: string };

/** Pick black/white foreground by WCAG contrast against the label color. */
function chipStyle(color: string): ChipStyle | null {
  const hex = color.startsWith("#") ? color : `#${color}`;
  try {
    const fg =
      contrastRatio("#000000", hex) >= contrastRatio("#ffffff", hex) ? "#000000" : "#ffffff";
    return { backgroundColor: hex, color: fg, borderColor: hex };
  } catch {
    return null;
  }
}

/** Colored label pill used on issue/pull rows and detail sidebars. */
export function LabelChip(props: { label: LabelPublic; className?: string }) @{
  const style = chipStyle(props.label.color);
  <span
    className={cn(
      "inline-flex items-center whitespace-nowrap rounded-full border px-2 text-[12px] font-medium leading-[18px]",
      !style && "border-border bg-muted text-muted-foreground",
      props.className,
    )}
    style={(style ?? undefined) as ChipStyle | undefined}
    title={(props.label.description || props.label.name) as string}
  >
    {props.label.name as string}
  </span>;
}
