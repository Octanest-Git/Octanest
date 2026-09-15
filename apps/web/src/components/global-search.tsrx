import { Search } from "@octanejs/lucide";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

/** Global forge search placeholder — users, orgs, repos, commits, PRs, code. */
export const GLOBAL_SEARCH_LABEL =
  "Search users, organizations, repositories, commits, pull requests, and code";

export const GLOBAL_SEARCH_PLACEHOLDER = "Search users, orgs, repos, code…";

type GlobalSearchProps = {
  className?: string;
};

export function GlobalSearch({ className }: GlobalSearchProps) @{
  <div className={cn("relative min-w-0", className)}>
    <Search
      aria-hidden
      size={16}
      className="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-muted-foreground sm:left-3"
    />
    <Input
      className="w-full bg-muted/35 pl-8 text-[13px] sm:pl-9 sm:text-[14px]"
      placeholder={GLOBAL_SEARCH_PLACEHOLDER}
      disabled
      title="Coming soon — search users, orgs, repos, commits, pull requests, and code"
      aria-label={GLOBAL_SEARCH_LABEL}
    />
  </div>;
}
