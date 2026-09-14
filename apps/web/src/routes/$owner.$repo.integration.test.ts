import {
  cleanup,
  render,
  screen,
  waitFor,
} from "@octanejs/testing-library";
import { createElement } from "octane";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

/**
 * Code / tree / blob browse (D-15, D-17, D-25 / GIT-05 UI).
 */

const getMock = vi.fn();
const treeMock = vi.fn();
const blobMock = vi.fn();
const refsMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    repo: {
      get: (...args: unknown[]) => getMock(...args),
      tree: (...args: unknown[]) => treeMock(...args),
      blob: (...args: unknown[]) => blobMock(...args),
      refs: (...args: unknown[]) => refsMock(...args),
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
    useParams: () => ({ owner: "ada", repo: "hello" }),
    useLoaderData: () => undefined,
    Link: MockLink,
  };
});

beforeEach(() => {
  getMock.mockReset();
  treeMock.mockReset();
  blobMock.mockReset();
  refsMock.mockReset();
});

afterEach(cleanup);

describe("/{owner}/{repo} Code home (D-15, D-25)", () => {
  it(
    "empty repo Code home shows Quick setup (not tree)",
    async () => {
      getMock.mockResolvedValue({
        ok: true,
        data: {
          id: "r1",
          owner_id: "u1",
          owner_type: "user",
          owner_username: "ada",
          name: "hello",
          description: "",
          visibility: "public",
          default_branch: "main",
          updated_at: "2026-09-12T00:00:00Z",
        },
      });
      treeMock.mockResolvedValue({
        ok: true,
        data: {
          empty: true,
          ref: "main",
          path: "",
          entries: [],
        },
      });
      refsMock.mockResolvedValue({ ok: true, data: { refs: [] } });

      const { RepoCodeHome } = await import("./$owner.$repo.index");
      render(RepoCodeHome as never);

      await waitFor(() => {
        expect(screen.getByText("Quick setup")).toBeInTheDocument();
      });
      expect(screen.queryByText("src")).not.toBeInTheDocument();
      expect(screen.queryByText("Page not found")).not.toBeInTheDocument();
      expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute(
        "href",
        "/ada/hello/settings",
      );
    },
    20000,
  );

  it(
    "tree route lists dirs first when commits exist",
    async () => {
      getMock.mockResolvedValue({
        ok: true,
        data: {
          id: "r1",
          owner_id: "u1",
          owner_type: "user",
          owner_username: "ada",
          name: "hello",
          description: "A sample repo",
          visibility: "public",
          default_branch: "main",
          updated_at: "2026-09-12T00:00:00Z",
        },
      });
      treeMock.mockResolvedValue({
        ok: true,
        data: {
          empty: false,
          ref: "main",
          path: "",
          entries: [
            { mode: "100644", kind: "blob", oid: "a", name: "README.md" },
            { mode: "040000", kind: "tree", oid: "b", name: "src" },
            { mode: "100644", kind: "blob", oid: "c", name: "package.json" },
          ],
        },
      });
      blobMock.mockResolvedValue({
        ok: true,
        data: {
          path: "README.md",
          ref: "main",
          size: 12,
          truncated: false,
          is_binary: false,
          encoding: "utf-8",
          content: "# Hello\n",
          soft_max_bytes: 1048576,
        },
      });
      refsMock.mockResolvedValue({
        ok: true,
        data: {
          refs: [{ name: "refs/heads/main", oid: "abc" }],
        },
      });

      const { RepoCodeHome } = await import("./$owner.$repo.index");
      render(RepoCodeHome as never);

      await waitFor(() => {
        expect(screen.getByText("src")).toBeInTheDocument();
      });
      expect(screen.queryByText("Quick setup")).not.toBeInTheDocument();

      const names = [
        "src",
        "README.md",
        "package.json",
      ].map((n) => screen.getByRole("link", { name: n }));
      const labels = names.map((el) => el.textContent?.trim() ?? "");
      // Re-query in document order via tree list links
      const treeLinks = screen
        .getByRole("list", { name: /directories|files|tree/i })
        .querySelectorAll("a");
      const ordered = [...treeLinks].map((a) => a.textContent?.trim() ?? "");
      expect(ordered[0]).toBe("src");
      expect(ordered).toContain("README.md");
      expect(ordered).toContain("package.json");
      expect(ordered.indexOf("src")).toBeLessThan(ordered.indexOf("README.md"));
      expect(labels).toContain("src");
    },
    20000,
  );

  it(
    "private non-owner (or missing) shows Not found — same copy",
    async () => {
      getMock.mockResolvedValue({
        ok: false,
        error: { code: "repo.not_found", message: "Repository not found" },
      });

      const { RepoCodeHome } = await import("./$owner.$repo.index");
      render(RepoCodeHome as never);

      await waitFor(() => {
        expect(screen.getByText("Page not found")).toBeInTheDocument();
      });
      expect(
        screen.getByText("We couldn't find that page."),
      ).toBeInTheDocument();
      expect(screen.queryByText("Quick setup")).not.toBeInTheDocument();
      expect(screen.queryByText("ada / hello")).not.toBeInTheDocument();
    },
    20000,
  );
});
