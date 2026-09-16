import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

const searchMock = vi.fn();
const getMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    repo: {
      get: (...args: unknown[]) => getMock(...args),
      search: (...args: unknown[]) => searchMock(...args),
    },
  },
}));

const navigateMock = vi.fn();

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@octanejs/tanstack-router")>();
  return {
    ...actual,
    useParams: () => ({ owner: "ada", repo: "hello" }),
    useSearch: () => ({ q: "UNIQUE_HIT", type: "code" }),
    useNavigate: () => navigateMock,
  };
});

import { RepoSearchPage } from "./$owner.$repo.search";

afterEach(() => {
  cleanup();
  searchMock.mockReset();
  getMock.mockReset();
  navigateMock.mockReset();
});

beforeEach(() => {
  searchMock.mockResolvedValue({
    ok: true,
    data: {
      type: "code",
      q: "UNIQUE_HIT",
      truncated: false,
      offset: 0,
      limit: 30,
      hits: [
        {
          kind: "code",
          path: "src/needle.txt",
          line: 2,
          content: "UNIQUE_HIT line",
        },
      ],
    },
  });
});

describe("repo search route (GIT-18 / D-SRCH-02 / D-SRCH-15)", () => {
  it("renders query input for /{owner}/{repo}/search", async () => {
    renderWithQueryClient(RepoSearchPage);
    expect(await screen.findByLabelText(/search query/i)).toBeTruthy();
  });

  it("shows code hits from repo.search type=code", async () => {
    renderWithQueryClient(RepoSearchPage);
    await waitFor(() => {
      expect(searchMock).toHaveBeenCalled();
    });
    expect(await screen.findByText(/src\/needle\.txt/)).toBeTruthy();
    expect(screen.getByText(/UNIQUE_HIT line/)).toBeTruthy();
  });

  it.todo("exposes type tabs: Code, Commits, Issues, Pull requests");

  it.todo("switches type query param and refetches on tab change");

  it.todo("renders empty and truncated states");
});
