import { Skeleton } from "@/components/ui/skeleton";

/** Signed-in home at `/` — mirrors {@link SignedInHome} geometry. */
export function SignedInHomeSkeleton() @{
  <div
    className="mx-auto max-w-3xl px-4 py-16"
    role="status"
    aria-busy="true"
    aria-label="Loading"
  >
    <Skeleton className="h-8 w-64 max-w-full" />
    <Skeleton className="mt-3 h-5 w-full max-w-md" />
    <Skeleton className="mt-2 h-5 w-72 max-w-full" />
    <div className="mt-8 flex flex-wrap items-start gap-3">
      <div className="flex w-full max-w-xs flex-col gap-2">
        <Skeleton className="h-11 w-full" />
        <Skeleton className="h-4 w-48" />
      </div>
      <Skeleton className="h-11 w-24" />
      <Skeleton className="h-11 w-20" />
    </div>
  </div>
}

export function ProfilePageSkeleton() @{
  <div
    className="mx-auto max-w-xl px-4 py-16"
    role="status"
    aria-busy="true"
    aria-label="Loading profile"
  >
    <Skeleton className="h-8 w-40" />
    <Skeleton className="mt-2 h-5 w-64" />
    <div className="mt-8 flex items-center gap-4">
      <Skeleton className="size-16 rounded-full" />
      <Skeleton className="h-11 w-36" />
    </div>
    <div className="mt-8 flex flex-col gap-4">
      <div className="flex flex-col gap-2">
        <Skeleton className="h-4 w-24" />
        <Skeleton className="h-11 w-full" />
      </div>
      <div className="flex flex-col gap-2">
        <Skeleton className="h-4 w-20" />
        <Skeleton className="h-11 w-full" />
      </div>
      <div className="flex flex-col gap-2">
        <Skeleton className="h-4 w-12" />
        <Skeleton className="h-24 w-full" />
      </div>
      <Skeleton className="h-11 w-28" />
    </div>
  </div>
}

export function AdminAuthSkeleton() @{
  <div
    className="mx-auto max-w-2xl px-4 py-16"
    role="status"
    aria-busy="true"
    aria-label="Loading auth settings"
  >
    <Skeleton className="h-8 w-48" />
    <Skeleton className="mt-2 h-5 w-full max-w-lg" />
    <div className="mt-8 flex flex-col gap-6">
      <div className="flex flex-col gap-2">
        <Skeleton className="h-4 w-28" />
        <Skeleton className="h-11 w-full" />
      </div>
      <div className="flex flex-col gap-2">
        <Skeleton className="h-4 w-32" />
        <Skeleton className="h-11 w-full" />
      </div>
      <Skeleton className="h-24 w-full" />
      <Skeleton className="h-11 w-36" />
    </div>
  </div>
}

export function StatusPageSkeleton() @{
  <div
    className="mx-auto max-w-3xl px-4 py-16"
    role="status"
    aria-busy="true"
    aria-label="Checking status"
  >
    <Skeleton className="h-8 w-48" />
    <div className="mt-8 rounded-lg border border-border bg-card p-6">
      <Skeleton className="h-5 w-40" />
      <Skeleton className="mt-4 h-4 w-56" />
      <Skeleton className="mt-2 h-4 w-44" />
    </div>
  </div>
}

/** Form-shaped placeholders inside {@link AuthShell}. */
export function AuthFormSkeleton({ fields = 2 }: { fields?: number }) @{
  const fieldIndexes = Array.from({ length: fields }, (_, i) => i);

  <div
    className="flex flex-col gap-4"
    role="status"
    aria-busy="true"
    aria-label="Loading form"
  >
    @for (const i of fieldIndexes; key i) {
      <div className="flex flex-col gap-2">
        <Skeleton className="h-4 w-28" />
        <Skeleton className="h-11 w-full" />
      </div>
    }
    <Skeleton className="h-11 w-full" />
    <Skeleton className="mx-auto h-4 w-40" />
  </div>
}
