import { createFileRoute } from "@octanejs/tanstack-router";
import { useEffect } from "octane";
import { Button, buttonVariants } from "@/components/ui/button";
import { OctanestMark } from "@/components/octanest-mark";
import { cn } from "@/lib/utils";

export const Route = createFileRoute("/")({
  component: LandingPage,
});

function revealExploreTarget() {
  const el = document.getElementById("explore");
  if (!el) return;
  // Finish translateY before hash scroll so middle/wheel landing doesn’t fight the reveal.
  el.classList.add("oct-reveal-in");
  el.style.transition = "none";
  // Re-enable transitions for later (no-op if already revealed).
  requestAnimationFrame(() => {
    el.style.transition = "";
  });
}

function useScrollReveal() {
  useEffect(() => {
    const nodes = Array.from(document.querySelectorAll<HTMLElement>(".oct-reveal"));
    if (typeof IntersectionObserver === "undefined") {
      nodes.forEach((n) => n.classList.add("oct-reveal-in"));
      return;
    }

    if (location.hash === "#explore") revealExploreTarget();

    const io = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          entry.target.classList.add("oct-reveal-in");
          io.unobserve(entry.target);
        }
      },
      { threshold: 0, rootMargin: "0px 0px 20% 0px" },
    );
    nodes.forEach((n) => io.observe(n));
    return () => io.disconnect();
  }, []);
}

function LandingPage() {
  useScrollReveal();

  return (
    <>
      {/* Avoid 100svh — mobile URL chrome resizes it and makes scroll feel broken. */}
      <section className="relative min-h-[32rem] overflow-hidden md:min-h-[36rem] lg:min-h-[40rem]">
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_12%_18%,color-mix(in_srgb,var(--primary)_42%,transparent),transparent_52%),radial-gradient(ellipse_at_88%_22%,color-mix(in_srgb,var(--secondary)_34%,transparent),transparent_48%),radial-gradient(ellipse_at_50%_100%,color-mix(in_srgb,var(--card)_80%,transparent),transparent_55%),linear-gradient(165deg,var(--background),color-mix(in_srgb,var(--card)_70%,var(--background)))]"
        />
        <div className="relative mx-auto flex min-h-[32rem] max-w-6xl flex-col justify-center px-4 py-14 sm:py-16 md:min-h-[36rem] lg:min-h-[40rem]">
          <div className="oct-rise" style={{ animationDelay: "0ms" }}>
            <OctanestMark size={96} />
          </div>
          <h1
            className="oct-rise mt-6 max-w-3xl font-[family-name:var(--font-display)] text-[clamp(1.75rem,5vw,2.5rem)] font-semibold leading-[1.15] text-foreground"
            style={{ animationDelay: "80ms" }}
          >
            Where repositories nest — cloud or yours
          </h1>
          <div className="oct-rise" style={{ animationDelay: "160ms" }}>
            <p className="mt-4 max-w-2xl text-[16px] text-muted-foreground">
              Social coding forge: host git, collaborate, and ship — one product
              for Octanest Cloud and self-host
            </p>
            <div className="mt-8 flex flex-wrap gap-3">
              <Button disabled title="Coming soon">
                Get started
              </Button>
              <a
                href="#explore"
                className={cn(buttonVariants({ variant: "secondary" }))}
                onClick={revealExploreTarget}
              >
                Explore
              </a>
            </div>
          </div>
        </div>
      </section>

      <section id="explore" className="scroll-mt-32 py-24 oct-reveal md:scroll-mt-20">
        <div className="mx-auto grid max-w-6xl gap-12 px-4 md:grid-cols-2">
          <div className="border-l-[3px] border-primary pl-6">
            <h2 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
              Octanest Cloud
            </h2>
            <p className="mt-4 text-[16px] text-muted-foreground">
              Hosted forge with the same product you run yourself — one release train.
            </p>
          </div>
          <div className="border-l-[3px] border-secondary pl-6">
            <h2 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
              Self-host
            </h2>
            <p className="mt-4 text-[16px] text-muted-foreground">
              Run Octanest on your machines with Docker Compose — your data, your network.
            </p>
          </div>
        </div>
      </section>

      <section className="py-24 oct-reveal">
        <div className="mx-auto grid max-w-6xl gap-12 px-4 md:grid-cols-3">
          <div>
            <h3 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
              Host git
            </h3>
            <p className="mt-4 text-[16px] text-muted-foreground">
              Browse repos and history in the browser.
            </p>
          </div>
          <div>
            <h3 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
              Collaborate
            </h3>
            <p className="mt-4 text-[16px] text-muted-foreground">
              Issues and pull requests for teams.
            </p>
          </div>
          <div>
            <h3 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
              Ship
            </h3>
            <p className="mt-4 text-[16px] text-muted-foreground">
              Releases and CI when you're ready to ship.
            </p>
          </div>
        </div>
      </section>

      <section className="border-t border-border bg-card/50 py-24">
        <div className="mx-auto max-w-6xl px-4">
          <h2 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
            One forge. Cloud or yours.
          </h2>
          <div className="mt-8 flex flex-wrap gap-3">
            <Button disabled title="Coming soon">
              Get started
            </Button>
            <a
              href="#explore"
              className={cn(buttonVariants({ variant: "secondary" }))}
              onClick={revealExploreTarget}
            >
              Explore
            </a>
          </div>
        </div>
      </section>

      <style>{`
        @keyframes octRise {
          from { opacity: 0; transform: translateY(8px); }
          to   { opacity: 1; transform: translateY(0); }
        }
        .oct-rise { animation: octRise 320ms ease-out both; }
        .oct-reveal {
          transition: opacity 320ms ease-out, transform 320ms ease-out;
        }
        .oct-reveal:not(.oct-reveal-in) {
          opacity: 0;
          transform: translateY(12px);
        }
        .oct-reveal-in {
          opacity: 1;
          transform: translateY(0);
        }
        @media (prefers-reduced-motion: reduce) {
          .oct-rise { animation: none !important; }
          .oct-reveal, .oct-reveal-in {
            opacity: 1 !important;
            transform: none !important;
            transition: none !important;
          }
        }
      `}</style>
    </>
  );
}
