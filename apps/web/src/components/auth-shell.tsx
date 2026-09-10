import { OctanestMark } from "@/components/octanest-mark";
import { cn } from "@/lib/utils";

export function AuthShell({
  title,
  support,
  children,
  className,
}: {
  title: string;
  support: string;
  children?: unknown;
  className?: string;
}) {
  return (
    <div className="relative overflow-hidden px-4 py-16">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_20%_10%,color-mix(in_srgb,var(--primary)_28%,transparent),transparent_50%),radial-gradient(ellipse_at_85%_20%,color-mix(in_srgb,var(--secondary)_22%,transparent),transparent_48%)]"
      />
      <div
        className={cn(
          "oct-auth-enter relative mx-auto flex w-full max-w-md flex-col",
          className,
        )}
      >
        <OctanestMark size={48} />
        <h1 className="mt-8 font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
          {title}
        </h1>
        <p className="mt-2 text-[16px] text-muted-foreground">{support}</p>
        <div className="mt-8 flex flex-col gap-4">{children}</div>
      </div>
      <style>{`
        @keyframes octAuthEnter {
          from { opacity: 0; transform: translateY(8px); }
          to   { opacity: 1; transform: translateY(0); }
        }
        .oct-auth-enter { animation: octAuthEnter 250ms ease-out both; }
        .oct-auth-error { animation: octAuthError 150ms ease-out both; }
        @keyframes octAuthError {
          from { opacity: 0; }
          to   { opacity: 1; }
        }
        @media (prefers-reduced-motion: reduce) {
          .oct-auth-enter, .oct-auth-error { animation: none !important; }
        }
      `}</style>
    </div>
  );
}

export function AuthErrorBanner({ message }: { message: string }) {
  if (!message) return null;
  return (
    <p
      role="alert"
      className="oct-auth-error text-[14px] font-normal leading-[1.4] text-destructive"
    >
      {message}
    </p>
  );
}
