import { createFileRoute, Link } from "@octanejs/tanstack-router";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/")({
  component: LandingPage,
});

function LandingPage() {
  return (
    <section className="relative overflow-hidden">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_20%_20%,color-mix(in_srgb,var(--color-accent-cool)_35%,transparent),transparent_55%),radial-gradient(ellipse_at_80%_30%,color-mix(in_srgb,var(--color-accent-warm)_30%,transparent),transparent_50%),linear-gradient(180deg,var(--color-bg),var(--color-surface))]"
      />
      <div className="relative mx-auto flex min-h-[calc(100vh-8rem)] max-w-6xl flex-col justify-center px-4 py-16 motion-safe:animate-[fadeIn_0.4s_ease-out]">
        <p className="font-[family-name:var(--font-display)] text-[40px] font-semibold leading-[1.15] tracking-tight text-[var(--color-text)]">
          Octanest
        </p>
        <h1 className="mt-4 max-w-3xl font-[family-name:var(--font-display)] text-[40px] font-semibold leading-[1.15] text-[var(--color-text)]">
          Where repositories nest — cloud or yours
        </h1>
        <p className="mt-4 max-w-2xl text-[16px] text-[var(--color-muted)]">
          Social coding forge: host git, collaborate, and ship — one product for
          Octanest Cloud and self-host
        </p>
        <div className="mt-8 flex flex-wrap gap-3">
          <Button
            onClick={() => {
              /* auth placeholder */
            }}
          >
            Get started
          </Button>
          <Link to="/" hash="explore">
            <Button variant="secondary">Explore Octanest</Button>
          </Link>
        </div>
      </div>
      <style>{`
        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(8px); }
          to { opacity: 1; transform: translateY(0); }
        }
        @media (prefers-reduced-motion: reduce) {
          .motion-safe\\:animate-\\[fadeIn_0\\.4s_ease-out\\] { animation: none !important; }
        }
      `}</style>
    </section>
  );
}
