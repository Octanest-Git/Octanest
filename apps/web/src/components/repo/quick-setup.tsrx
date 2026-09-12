import { Link } from "@octanejs/tanstack-router";

type QuickSetupProps = {
  owner: string;
  repo: string;
};

/** Empty-repo first-push guide (D-15 / UI-SPEC Quick setup). */
export function QuickSetup({ owner, repo }: QuickSetupProps) @{
  const origin =
    typeof window !== "undefined" ? window.location.origin : "";
  const httpsUrl = `${origin}/${owner}/${repo}.git`;
  const remoteCmd = `git remote add origin ${httpsUrl}`;
  const pushCmd = `git push -u origin HEAD`;
  const setupScript = `echo "# ${repo}" >> README.md
git init
git add README.md
git commit -m "first commit"
${remoteCmd}
${pushCmd}`;

  <section
    className="rounded-md border border-border bg-card/30 px-6 py-8"
    aria-labelledby="quick-setup-heading"
  >
    <h2
      id="quick-setup-heading"
      className="font-[family-name:var(--font-display)] text-[20px] font-semibold leading-[1.3] text-foreground"
    >
      Quick setup
    </h2>
    <p className="mt-2 text-[16px] text-muted-foreground">
      This repository is empty. Push an existing repository from the command
      line using the HTTPS remote below.
    </p>

    <div className="mt-6 flex flex-col gap-4">
      <div className="flex flex-col gap-2">
        <p className="text-[14px] font-normal text-muted-foreground">HTTPS</p>
        <code className="block overflow-x-auto rounded-md border border-border bg-muted/40 px-3 py-2 font-mono text-[13px] text-foreground">
          {httpsUrl as string}
        </code>
      </div>

      <div className="flex flex-col gap-2">
        <p className="text-[14px] font-normal text-muted-foreground">
          …or create a new repository on the command line
        </p>
        <pre className="overflow-x-auto rounded-md border border-border bg-muted/40 px-3 py-3 font-mono text-[13px] leading-[1.5] text-foreground whitespace-pre-wrap">
          {setupScript as string}
        </pre>
      </div>

      <p className="text-[14px] text-muted-foreground">
        Prefer starting from the form?{" "}
        <Link
          to="/new"
          preload="intent"
          className="font-semibold text-primary underline-offset-4 hover:underline"
        >
          Create another repository
        </Link>
      </p>
    </div>
  </section>
}
