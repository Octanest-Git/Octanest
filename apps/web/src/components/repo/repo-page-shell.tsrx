/** Shared fixed-width shell for all `/{owner}/{repo}` surfaces (Code chrome width). */
export function RepoPageShell({
  children,
  className,
}: {
  children?: unknown;
  className?: string;
}) @{
  <div
    className={
      (className
        ? `mx-auto w-full max-w-6xl px-4 py-10 ${className}`
        : "mx-auto w-full max-w-6xl px-4 py-10") as string
    }
  >
    {children}
  </div>
}
