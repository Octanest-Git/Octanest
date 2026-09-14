import { createElement } from "octane";
import {
  cleanup,
  render,
  screen,
  waitFor,
} from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

/**
 * Phase 11 Issues UI Wave 0 / Nyquist stubs (ISS-01..04, D-ISS-10/13/16/19, D-ISS-02/04/07/11).
 *
 * Prefer `it.fails` so remaining RED cases do not fail the suite until greened in
 * 11-03 / 11-04 / 11-05 / 11-06 / 11-08 / 11-09. Do not author production .tsrx here.
 *
 * Runtime-variable dynamic import with @vite-ignore keeps Vitest collectable while
 * routes are still absent (Phase 08/10 Wave 0 pattern).
 */

const getMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    repo: {
      get: (...args: unknown[]) => getMock(...args),
    },
  },
}));

vi.mock("@/lib/use-chrome-account", () => ({
  useChromeAccountState: () => ({
    pending: false,
    user: {
      id: "u1",
      email: "ada@example.com",
      username: "ada",
      display_name: "Ada",
      bio: "",
      role: "user",
      profile_incomplete: false,
      email_verified: true,
      must_change_credentials: false,
    },
    needsSetup: false,
    allowSignup: true,
  }),
  resolveAllowSignup: () => true,
}));

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("@octanejs/tanstack-router")>();
  function MockLink(props: {
    to?: string;
    href?: string;
    children?: unknown;
    className?: string;
    preload?: string;
  }) {
    return createElement(
      "a",
      {
        href: (props.href ?? props.to ?? "#") as string,
        className: props.className,
      } as never,
      props.children as never,
    );
  }
  return {
    ...actual,
    useParams: () => ({ owner: "ada", repo: "hello", n: "1" }),
    useLoaderData: () => undefined,
    Link: MockLink,
  };
});

const readableRepo = {
  id: "r1",
  owner_id: "u1",
  owner_type: "user" as const,
  owner_username: "ada",
  name: "hello",
  description: "",
  visibility: "public" as const,
  default_branch: "main",
  updated_at: "2026-09-14T00:00:00Z",
  can_admin: true,
  can_write: true,
};

beforeEach(() => {
  getMock.mockReset();
  getMock.mockResolvedValue({ ok: true, data: readableRepo });
});

afterEach(cleanup);

async function loadIssuesListModule(): Promise<Record<string, unknown>> {
  const rel = "./$owner.$repo.issues";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /{owner}/{repo}/issues missing — implement in 11-03 (D-ISS-16 / D-ISS-19). ${(err as Error).message}`,
    );
  }
}

async function loadIssuesNewModule(): Promise<Record<string, unknown>> {
  const rel = "./$owner.$repo.issues.new";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /{owner}/{repo}/issues/new missing — implement in 11-03 (D-ISS-19 / D-ISS-10). ${(err as Error).message}`,
    );
  }
}

async function loadIssueDetailModule(): Promise<Record<string, unknown>> {
  const rel = "./$owner.$repo.issues.$n";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /{owner}/{repo}/issues/{n} missing — implement in 11-03/11-04 (ISS-01..04). ${(err as Error).message}`,
    );
  }
}

function issuesListPage(mod: Record<string, unknown>): unknown {
  const page = mod.IssuesListPage ?? mod.IssuesPage ?? mod.default;
  expect(
    page,
    "Wave 0: IssuesListPage (or IssuesPage) must be exported from $owner.$repo.issues",
  ).toBeTruthy();
  return page;
}

function issuesNewPage(mod: Record<string, unknown>): unknown {
  const page = mod.IssuesNewPage ?? mod.NewIssuePage ?? mod.default;
  expect(
    page,
    "Wave 0: IssuesNewPage must be exported from $owner.$repo.issues.new",
  ).toBeTruthy();
  return page;
}

function issueDetailPage(mod: Record<string, unknown>): unknown {
  const page = mod.IssueDetailPage ?? mod.IssuesDetailPage ?? mod.default;
  expect(
    page,
    "Wave 0: IssueDetailPage must be exported from $owner.$repo.issues.$n",
  ).toBeTruthy();
  return page;
}

describe("repo chrome Issues tab Wave 0 (D-ISS-19)", () => {
  it.fails(
    "Issues tab present for readable repos with href /{owner}/{repo}/issues",
    async () => {
      const { RepoChrome } = await import("../components/repo/repo-chrome");
      render(
        createElement(RepoChrome as never, {
          repo: readableRepo,
          active: "code",
        }) as never,
      );

      await waitFor(() => {
        expect(
          screen.getByRole("link", { name: "Issues" }),
        ).toBeInTheDocument();
      });
      expect(screen.getByRole("link", { name: "Issues" })).toHaveAttribute(
        "href",
        "/ada/hello/issues",
      );
    },
    15_000,
  );
});

describe("/{owner}/{repo}/issues list Wave 0 (D-ISS-16 / D-ISS-19)", () => {
  it.fails(
    "list defaults Open with Closed and All controls",
    async () => {
      const mod = await loadIssuesListModule();
      const { container } = renderWithQueryClient(issuesListPage(mod));

      await waitFor(() => {
        expect(
          screen.getByRole("tab", { name: /^Open$/i }) ??
            screen.getByRole("link", { name: /^Open$/i }) ??
            screen.getByText(/^Open$/i),
        ).toBeTruthy();
      });
      expect(screen.getByText(/^Closed$/i)).toBeInTheDocument();
      expect(screen.getByText(/^All$/i)).toBeInTheDocument();
      // Open is the default selection (tab/link aria-current or pressed)
      const openControl =
        container.querySelector('[aria-current="page"]') ??
        container.querySelector('[aria-selected="true"]') ??
        container.querySelector('[data-state="active"]');
      expect(openControl?.textContent).toMatch(/Open/i);
    },
    15_000,
  );

  it.fails(
    "New issue from list when can_write (D-ISS-19 / D-ISS-20)",
    async () => {
      getMock.mockResolvedValue({
        ok: true,
        data: { ...readableRepo, can_write: true, can_admin: false },
      });
      const mod = await loadIssuesListModule();
      renderWithQueryClient(issuesListPage(mod));

      await waitFor(() => {
        expect(
          screen.getByRole("link", { name: /New issue/i }),
        ).toBeInTheDocument();
      });
      expect(screen.getByRole("link", { name: /New issue/i })).toHaveAttribute(
        "href",
        "/ada/hello/issues/new",
      );
    },
    15_000,
  );

  it.fails(
    "New issue hidden when !can_write (empty state still readable)",
    async () => {
      getMock.mockResolvedValue({
        ok: true,
        data: { ...readableRepo, can_write: false, can_admin: false },
      });
      const mod = await loadIssuesListModule();
      renderWithQueryClient(issuesListPage(mod));

      await waitFor(() => {
        expect(screen.getByText(/No open issues|No issues/i)).toBeInTheDocument();
      });
      expect(
        screen.queryByRole("link", { name: /New issue/i }),
      ).not.toBeInTheDocument();
      expect(
        screen.queryByRole("button", { name: /New issue/i }),
      ).not.toBeInTheDocument();
    },
    15_000,
  );

  it.fails(
    "Admin label settings entry gated by can_admin (D-ISS-07)",
    async () => {
      getMock.mockResolvedValue({
        ok: true,
        data: { ...readableRepo, can_write: true, can_admin: true },
      });
      const mod = await loadIssuesListModule();
      renderWithQueryClient(issuesListPage(mod));

      await waitFor(() => {
        expect(
          screen.getByRole("link", { name: /Labels|Manage labels/i }),
        ).toBeInTheDocument();
      });

      getMock.mockResolvedValue({
        ok: true,
        data: { ...readableRepo, can_write: true, can_admin: false },
      });
      cleanup();
      const mod2 = await loadIssuesListModule();
      renderWithQueryClient(issuesListPage(mod2));
      await waitFor(() => {
        expect(screen.getByText(/^Open$/i)).toBeInTheDocument();
      });
      expect(
        screen.queryByRole("link", { name: /Labels|Manage labels/i }),
      ).not.toBeInTheDocument();
    },
    15_000,
  );
});

describe("/{owner}/{repo}/issues/new Wave 0 (D-ISS-10)", () => {
  it.fails(
    "Write|Preview tabs on new issue form",
    async () => {
      const mod = await loadIssuesNewModule();
      renderWithQueryClient(issuesNewPage(mod));

      await waitFor(() => {
        expect(screen.getByText(/^Write$/i)).toBeInTheDocument();
      });
      expect(screen.getByText(/^Preview$/i)).toBeInTheDocument();
    },
    15_000,
  );
});

describe("/{owner}/{repo}/issues/{n} detail Wave 0 (ISS-01..04 / D-ISS-13)", () => {
  it.fails(
    "detail shows title/body/comments/labels/assignees + Linked PRs panel shell",
    async () => {
      const mod = await loadIssueDetailModule();
      renderWithQueryClient(issueDetailPage(mod));

      await waitFor(() => {
        expect(
          screen.getByRole("heading", { level: 1 }) ??
            screen.getByTestId("issue-title"),
        ).toBeTruthy();
      });
      expect(screen.getByText(/Comments|comment/i)).toBeInTheDocument();
      expect(screen.getByText(/Labels/i)).toBeInTheDocument();
      expect(screen.getByText(/Assignees/i)).toBeInTheDocument();
      expect(screen.getByText(/Linked PRs/i)).toBeInTheDocument();
    },
    15_000,
  );

  it.fails(
    "Write|Preview on edit and comment forms (D-ISS-10)",
    async () => {
      const mod = await loadIssueDetailModule();
      renderWithQueryClient(issueDetailPage(mod));

      await waitFor(() => {
        const writes = screen.getAllByText(/^Write$/i);
        const previews = screen.getAllByText(/^Preview$/i);
        expect(writes.length).toBeGreaterThanOrEqual(1);
        expect(previews.length).toBeGreaterThanOrEqual(1);
      });
    },
    15_000,
  );

  it.fails(
    "lifecycle affordances edit/close/reopen + history panel + Admin delete confirm (D-ISS-02 / D-ISS-04) — green in 11-04",
    async () => {
      const mod = await loadIssueDetailModule();
      renderWithQueryClient(issueDetailPage(mod));

      await waitFor(() => {
        expect(
          screen.getByRole("button", { name: /Edit|Close issue|Reopen/i }),
        ).toBeInTheDocument();
      });
      expect(screen.getByText(/History|Edit history/i)).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: /Delete issue/i }),
      ).toBeInTheDocument();
      // confirmNumber dialog copy appears after Delete (Admin)
      screen.getByRole("button", { name: /Delete issue/i }).click();
      await waitFor(() => {
        expect(
          screen.getByText(/type.*(issue )?number|confirm/i),
        ).toBeInTheDocument();
      });
    },
    15_000,
  );

  it.fails(
    "reaction bar visibility + Write+ toggle on issue and comments (D-ISS-11) — green in 11-08",
    async () => {
      const mod = await loadIssueDetailModule();
      renderWithQueryClient(issueDetailPage(mod));

      await waitFor(() => {
        expect(
          screen.getByRole("toolbar", { name: /[Rr]eactions?/ }) ??
            screen.getByLabelText(/[Rr]eactions?/),
        ).toBeTruthy();
      });
      // Eight GitHub contents represented somehow in the bar
      expect(document.body.textContent).toMatch(/\+1|👍|react/i);
    },
    15_000,
  );
});
