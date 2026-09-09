import { Search } from "@octanejs/lucide";
import { useEffect, useState } from "octane";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

/** Global forge search placeholder — users, orgs, repos, commits, PRs, code. */
export const GLOBAL_SEARCH_LABEL =
  "Search users, organizations, repositories, commits, pull requests, and code";

export const GLOBAL_SEARCH_PLACEHOLDER = "Search users, orgs, repos, code…";

/** Shorter label for narrow one-row headers; full copy stays in aria-label/title. */
export const GLOBAL_SEARCH_PLACEHOLDER_COMPACT = "Search…";

type GlobalSearchProps = {
  className?: string;
};

export function GlobalSearch({ className }: GlobalSearchProps) {
  const [compact, setCompact] = useState(false);

  useEffect(() => {
    const mq = window.matchMedia("(max-width: 639px)");
    const sync = () => setCompact(mq.matches);
    sync();
    mq.addEventListener("change", sync);
    return () => mq.removeEventListener("change", sync);
  }, []);

  return (
    <div className={cn("relative min-w-0", className)}>
      <Search
        aria-hidden
        size={16}
        className="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-muted-foreground sm:left-3"
      />
      <Input
        className="w-full bg-muted/35 pl-8 text-[13px] sm:pl-9 sm:text-[14px]"
        placeholder={
          compact ? GLOBAL_SEARCH_PLACEHOLDER_COMPACT : GLOBAL_SEARCH_PLACEHOLDER
        }
        disabled
        title="Coming soon — search users, orgs, repos, commits, pull requests, and code"
        aria-label={GLOBAL_SEARCH_LABEL}
      />
    </div>
  );
}
