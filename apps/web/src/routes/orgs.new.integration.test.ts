import { cleanup, fireEvent, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "@octanest/web/test-runner";
import { vi } from "vitest";

/**
 * ORG-01 / D-ORG-01 / D-ORG-06: /orgs/new create UI (10-13 tracer).
 */

const createMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(),
    },
    org: {
      create: (...args: unknown[]) => createMock(...args),
    },
  },
}));

type LoaderShape = {
  user: {
    id: string;
    email: string;
    username: string;
    display_name: string;
    bio: string;
    avatar_url: null;
    role: string;
    profile_incomplete: boolean;
    email_verified: boolean;
  };
};

let loaderData: LoaderShape;

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@octanejs/tanstack-router")>();
  return {
    ...actual,
    useLoaderData: () => loaderData,
  };
});

beforeEach(() => {
  createMock.mockReset();
  loaderData = {
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
  };
});

afterEach(cleanup);

describe("/orgs/new (ORG-01 / D-ORG-01 / D-ORG-06)", () => {
  it("verified: Slug + optional Display name + Create organization CTA", async () => {
    loaderData.user.email_verified = true;
    const { OrgsNewPage } = await import("./orgs.new");
    render(OrgsNewPage as never);

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Create a new organization" }),
      ).toBeInTheDocument();
    });
    expect(screen.getByLabelText("Slug")).toBeInTheDocument();
    expect(screen.getByLabelText("Display name (optional)")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Create organization" })).toBeInTheDocument();

    createMock.mockResolvedValueOnce({
      ok: true,
      data: {
        id: "o1",
        slug: "acme",
        display_name: "Acme",
        member_base_permission: "none",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      },
    });
    const assign = vi.spyOn(window.location, "assign").mockImplementation(() => {});

    fireEvent.input(screen.getByLabelText("Slug"), {
      target: { value: "acme" },
    });
    fireEvent.input(screen.getByLabelText("Display name (optional)"), {
      target: { value: "Acme" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create organization" }));

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledWith({
        slug: "acme",
        display_name: "Acme",
      });
    });
    await waitFor(() => {
      expect(assign).toHaveBeenCalledWith("/acme");
    });
    assign.mockRestore();
  });

  it("unverified: Verify your email wall — not the create form", async () => {
    const { OrgsNewPage } = await import("./orgs.new");
    render(OrgsNewPage as never);

    await waitFor(() => {
      expect(screen.getByText("Verify your email")).toBeInTheDocument();
    });
    expect(
      screen.getByText("Verify your email before creating an organization."),
    ).toBeInTheDocument();
    expect(screen.queryByLabelText("Slug")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Create organization" })).not.toBeInTheDocument();
  });

  it("reserved slug shows That username is reserved. Choose a different username.", async () => {
    loaderData.user.email_verified = true;
    const { OrgsNewPage } = await import("./orgs.new");
    render(OrgsNewPage as never);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Create organization" })).toBeInTheDocument();
    });

    createMock.mockResolvedValueOnce({
      ok: false,
      error: { code: "auth.reserved_username", message: "username is reserved" },
    });

    fireEvent.input(screen.getByLabelText("Slug"), {
      target: { value: "orgs" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create organization" }));

    await waitFor(() => {
      expect(
        screen.getByText("That username is reserved. Choose a different username."),
      ).toBeInTheDocument();
    });
  });

  it("taken slug shows slug already used by a user or org", async () => {
    loaderData.user.email_verified = true;
    const { OrgsNewPage } = await import("./orgs.new");
    render(OrgsNewPage as never);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Create organization" })).toBeInTheDocument();
    });

    createMock.mockResolvedValueOnce({
      ok: false,
      error: {
        code: "org.slug_taken",
        message: "That slug is already used by a user or organization. Choose a different slug.",
      },
    });

    fireEvent.input(screen.getByLabelText("Slug"), {
      target: { value: "taken-slug" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create organization" }));

    await waitFor(() => {
      expect(
        screen.getByText(
          "That slug is already used by a user or organization. Choose a different slug.",
        ),
      ).toBeInTheDocument();
    });
  });
});
