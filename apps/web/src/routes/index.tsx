import { createFileRoute } from "@octanejs/tanstack-router";
import { Button, buttonVariants } from "@/components/ui/button";
import { OctanestMark } from "@/components/octanest-mark";
import { cn } from "@/lib/utils";

export const Route = createFileRoute("/")({
  component: LandingPage,
});

function LandingPage() {
  return (
    <>
      <section className="relative min-h-[calc(100vh-8rem)] overflow-hidden">
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_20%_20%,color-mix(in_srgb,var(--primary)_32%,transparent),transparent_55%),radial-gradient(ellipse_at_80%_30%,color-mix(in_srgb,var(--secondary)_26%,transparent),transparent_50%),linear-gradient(180deg,var(--background),var(--card))]"
        />
        <div className="relative mx-auto flex min-h-[calc(100vh-8rem)] max-w-6xl flex-col justify-center px-4 py-16">
          <OctanestMark size={96} />
          <h1 className="mt-6 max-w-3xl font-[family-name:var(--font-display)] text-[40px] font-semibold leading-[1.15] text-foreground">
            Where repositories nest — cloud or yours
          </h1>
          <p className="mt-4 max-w-2xl text-[16px] text-muted-foreground">
            Social coding forge: host git, collaborate, and ship — one product
            for Octanest Cloud and self-host
          </p>
          <div className="mt-8 flex flex-wrap gap-3">
            <Button disabled title="Coming soon">
              Get started
            </Button>
            <a href="#explore" className={cn(buttonVariants({ variant: "secondary" }))}>
              Explore
            </a>
          </div>
        </div>
      </section>

      <section id="explore" className="py-24">
        <div className="mx-auto grid max-w-6xl gap-12 px-4 md:grid-cols-2">
          <div className="border-l-2 border-primary pl-6">
            <h2 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
              Octanest Cloud
            </h2>
            <p className="mt-4 text-[16px] text-muted-foreground">
              Hosted forge with the same product you run yourself — one release train.
            </p>
          </div>
          <div className="border-l-2 border-secondary pl-6">
            <h2 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
              Self-host
            </h2>
            <p className="mt-4 text-[16px] text-muted-foreground">
              Run Octanest on your machines with Docker Compose — your data, your network.
            </p>
          </div>
        </div>
      </section>

      <section className="py-24">
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

      <section className="py-24">
        <div className="mx-auto max-w-6xl px-4">
          <h2 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
            One forge. Cloud or yours.
          </h2>
          <div className="mt-8 flex flex-wrap gap-3">
            <Button disabled title="Coming soon">
              Get started
            </Button>
            <a href="#explore" className={cn(buttonVariants({ variant: "secondary" }))}>
              Explore
            </a>
          </div>
        </div>
      </section>
    </>
  );
}
