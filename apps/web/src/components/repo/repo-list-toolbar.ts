/** Shared toolbar classes for repo issues / pulls list headers. */

export function repoListTabClass(active: boolean): string {
  return active
    ? "rounded-md bg-muted px-3 py-1.5 text-sm font-medium text-foreground"
    : "rounded-md px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground";
}

export function repoListFilterFieldClass(): string {
  return "min-w-[9rem] rounded-md border border-border bg-background px-2 py-1.5 text-sm text-foreground";
}

export function repoListFilterLabelClass(): string {
  return "flex min-w-[9rem] flex-col gap-1 text-sm";
}

export function repoListFilterFormClass(): string {
  return "flex flex-wrap items-end gap-3";
}

export function repoListTablistClass(): string {
  return "flex flex-wrap gap-2";
}

export function repoListApplyClass(): string {
  return "shrink-0 rounded-md border border-border px-3 py-1.5 text-sm font-medium";
}

export function repoListPrimaryCtaClass(): string {
  return "inline-flex items-center rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground no-underline hover:brightness-110";
}
