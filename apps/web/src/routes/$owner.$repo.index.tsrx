import {
  createFileRoute,
  Link,
  useParams,
} from "@octanejs/tanstack-router";

export const Route = createFileRoute("/$owner/$repo/")({
  component: RepoCodeHome,
  head: ({ params }) => ({
    meta: [
      {
        title: `${params.owner}/${params.repo} · Octanest`,
      },
    ],
  }),
});

/** Empty Code home — Quick setup first-push guide (D-10, D-15). Full tree in 07-15. */
export function RepoCodeHome() @{
  const params = useParams({ from: "/$owner/$repo/" }) as {
    owner: string;
    repo: string;
  };
  const owner = params.owner ?? "";
  const repo = params.repo ?? "";
  const origin =
    typeof window !== "undefined" ? window.location.origin : "";
  const httpsUrl = `${origin}/${owner}/${repo}.git`;
  const remoteCmd = `git remote add origin ${httpsUrl}`;
  const pushCmd = `git push -u origin HEAD`;
  const title = `${owner} / ${repo}`;
  const setupScript = `echo "# ${repo}" >> README.md
git init
git add README.md
git commit -m "first commit"
${remoteCmd}
${pushCmd}`;

  <div className="mx-auto max-w-5xl px-4 py-10">
    <header className="mb-6">
      <h1 className="font-[family-name:var(--font-display)] text-[24px] font-semibold leading-[1.2] text-foreground">
        {title as string}
      </h1>
    </header>

    <nav
      className="mb-8 flex flex-wrap gap-4 border-b border-border pb-0"
      aria-label="Repository"
    >
      <span className="border-b-2 border-primary pb-2 text-[14px] font-semibold text-foreground">
        Code
      </span>
      <span className="pb-2 text-[14px] font-normal text-muted-foreground">
        Commits
      </span>
      <span className="pb-2 text-[14px] font-normal text-muted-foreground">
        Branches
      </span>
      <span className="pb-2 text-[14px] font-normal text-muted-foreground">
        Tags
      </span>
    </nav>

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
        This repository is empty. Create a local git repo, then push over HTTPS
        (authentication arrives in a later phase).
      </p>

      <div className="mt-6 flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          <p className="text-[14px] font-normal text-muted-foreground">
            HTTPS clone URL
          </p>
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
  </div>
}
