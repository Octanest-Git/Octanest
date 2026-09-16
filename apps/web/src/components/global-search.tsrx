import { Search } from "@octanejs/lucide";
import { useState } from "octane";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

/** Header search — discovers public repos via `/explore` (SOC-03). */
export const GLOBAL_SEARCH_LABEL = "Search public repositories";

export const GLOBAL_SEARCH_PLACEHOLDER = "Search public repositories…";

type GlobalSearchProps = {
  className?: string;
};

export function GlobalSearch({ className }: GlobalSearchProps) @{
  const [q, setQ] = useState("");

  function goExplore() {
    const next = q.trim();
    window.location.assign(next ? `/explore?q=${encodeURIComponent(next)}` : "/explore");
  }

  <form
    method="get"
    action="/explore"
    className={cn("relative min-w-0", className)}
    onSubmit={(e) => {
      e.preventDefault();
      goExplore();
    }}
  >
    <Search
      aria-hidden
      size={16}
      className="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-muted-foreground sm:left-3"
    />
    <Input
      name="q"
      className="w-full bg-muted/35 pl-8 text-[13px] sm:pl-9 sm:text-[14px]"
      placeholder={GLOBAL_SEARCH_PLACEHOLDER}
      value={q as string}
      onInput={(e) => setQ((e.currentTarget as HTMLInputElement).value)}
      title="Search public repositories on this instance"
      aria-label={GLOBAL_SEARCH_LABEL}
    />
  </form>;
}
