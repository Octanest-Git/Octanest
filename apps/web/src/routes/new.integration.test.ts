import { cleanup, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

/**
 * Wave 0 stubs for `/new` (D-01, D-11 / UI-SPEC Unverified `/new`).
 * Greened by 07-13 when `new.tsrx` ships the verify wall + create form.
 *
 * Dynamic id + @vite-ignore keeps the suite loadable while `./new` is absent
 * (Wave 0 RED). Static `import("./new")` fails Vite transform and yields 0 tests.
 */
vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(async () => ({
        ok: true,
        data: {
          id: "u1",
          email: "ada@example.com",
          username: "ada",
          display_name: "Ada",
          bio: "",
          avatar_url: null,
          role: "user",
          profile_incomplete: false,
          email_verified: false,
        },
      })),
    },
    repo: {
      create: vi.fn(),
    },
  },
}));

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@octanejs/tanstack-router")>();
  return {
    ...actual,
    useLoaderData: () => ({
      user: {
        id: "u1",
        email: "ada@example.com",
        username: "ada",
        display_name: "Ada",
        bio: "",
        avatar_url: null,
        role: "user",
        profile_incomplete: false,
        email_verified: false,
      },
    }),
  };
});

afterEach(cleanup);

describe("/new Wave 0 (D-11 verify wall)", () => {
  it("unverified session shows Verify your email wall — not the create form", async () => {
    const newRouteId = "./new";
    let mod: Record<string, unknown> | null = null;
    try {
      mod = (await import(/* @vite-ignore */ newRouteId)) as Record<
        string,
        unknown
      >;
    } catch {
      mod = null;
    }

    expect(
      mod,
      "apps/web/src/routes/new.tsrx must exist for /new integration tests (07-13)",
    ).toBeTruthy();
    expect(
      mod,
      "NewPage must be exported for integration tests (07-13)",
    ).toHaveProperty("NewPage");

    const { NewPage } = mod as { NewPage: unknown };
    render(NewPage as never);

    await waitFor(() => {
      expect(screen.getByText("Verify your email")).toBeInTheDocument();
    });
    expect(
      screen.getByText("Verify your email before creating a repository."),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Verify email" })).toHaveAttribute(
      "href",
      "/verify",
    );
    expect(
      screen.queryByRole("button", { name: "Create repository" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/repository name/i)).not.toBeInTheDocument();
  });
});
