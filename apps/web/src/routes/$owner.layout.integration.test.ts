/**
 * Regression: `/$owner` is a layout for repos/packages; org overview is index-only.
 * Parent must not notFound() for user accounts or `/{user}/{repo}` never renders.
 */
import { createElement } from "octane";
import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";
import type { OrgOverviewLoaderData } from "./$owner.index";

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@octanejs/tanstack-router")>();
  const loaderData: OrgOverviewLoaderData = {
    org: {
      id: "o1",
      slug: "acme",
      display_name: "Acme Corp",
      member_base_permission: "read",
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    },
    memberCount: 3,
    repos: [
      {
        id: "r1",
        owner_id: "o1",
        owner_type: "org",
        owner_username: "acme",
        name: "demo",
        description: "Demo repo",
        visibility: "public",
        default_branch: "main",
        updated_at: "2026-09-14T00:00:00Z",
        can_admin: true,
        can_write: true,
      },
    ],
    canAdmin: true,
  };
  return {
    ...actual,
    useParams: () => ({ owner: "acme" }),
    useLoaderData: () => loaderData,
    Link: (props: {
      to?: string;
      params?: Record<string, string>;
      children?: unknown;
      className?: string;
    }) => {
      const owner = props.params?.owner ?? "acme";
      const repo = props.params?.repo ?? "";
      const href = props.to?.includes("$repo") ? `/${owner}/${repo}` : (props.to ?? "#");
      return createElement(
        "a",
        { href, className: props.className } as never,
        props.children as never,
      );
    },
  };
});

import { OrgOverviewPage } from "./$owner.index";

afterEach(cleanup);

describe("/$owner layout vs org index", () => {
  it("layout has no org notFound loader; index owns org overview", async () => {
    const layout = await import("./$owner.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    const index = await import("./$owner.index.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(layout).toMatch(/Outlet/);
    expect(layout).not.toMatch(/notFound\(/);
    expect(layout).not.toMatch(/fetchOrgOverview/);
    expect(index).toMatch(/fetchOrgOverview/);
    expect(index).toMatch(/notFound\(/);
    expect(index).toMatch(/OrgOverviewPage/);
  }, 30_000);

  it("renders org overview shell for loader data (G-11.1-15)", async () => {
    renderWithQueryClient(OrgOverviewPage);

    await waitFor(
      () => {
        expect(screen.getByRole("heading", { name: "Acme Corp" })).toBeTruthy();
        expect(screen.getByText("@acme")).toBeTruthy();
        expect(screen.getByText("3 members")).toBeTruthy();
        expect(screen.getByRole("heading", { name: "Repositories" })).toBeTruthy();
        expect(screen.getByText("demo")).toBeTruthy();
      },
      { timeout: 10_000 },
    );
  });
});
