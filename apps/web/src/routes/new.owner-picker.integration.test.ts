import { cleanup, fireEvent, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "@octanest/web/test-runner";
import { vi } from "vitest";

/**
 * D-ORG-06: /new owner picker (self + Owner/Admin orgs).
 */

const createMock = vi.fn();
const listMineMock = vi.fn();

vi.mock("@/lib/spdx-licenses", () => ({
  listLicensePickerOptions: () => [],
  listSpdxLicenseOptions: () => [{ id: "none", label: "None" }],
}));

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(),
    },
    org: {
      listMine: (...args: unknown[]) => listMineMock(...args),
    },
    repo: {
      create: (...args: unknown[]) => createMock(...args),
      createDefaults: vi.fn(),
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
  defaults: {
    default_visibility: "public" | "private";
    stacks: never[];
    gitignores: never[];
  } | null;
  ownerOrgs: {
    id: string;
    slug: string;
    display_name: string;
    member_base_permission: "none" | "read" | "write";
    role: "owner" | "admin" | "member";
    created_at: string;
    updated_at: string;
  }[];
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
  listMineMock.mockReset();
  listMineMock.mockResolvedValue({
    ok: true,
    data: {
      orgs: [
        { slug: "acme", display_name: "Acme", role: "owner" },
        { slug: "widgets", display_name: "Widgets", role: "admin" },
        { slug: "readonly-co", display_name: "ReadOnly Co", role: "member" },
      ],
    },
  });
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
      email_verified: true,
    },
    defaults: {
      default_visibility: "public",
      stacks: [],
      gitignores: [],
    },
    // SSR already filters to Owner/Admin — Member-only orgs omitted (D-ORG-06).
    ownerOrgs: [
      {
        id: "o1",
        slug: "acme",
        display_name: "Acme",
        member_base_permission: "none",
        role: "owner",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      },
      {
        id: "o2",
        slug: "widgets",
        display_name: "Widgets",
        member_base_permission: "none",
        role: "admin",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      },
    ],
  };
});

afterEach(() => {
  cleanup();
  document.body.innerHTML = "";
});

async function loadNewModule(): Promise<Record<string, unknown>> {
  const rel = "./new";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /new route missing — owner picker needs NewPage (D-ORG-06). ${(err as Error).message}`,
    );
  }
}

function ownerTrigger(): HTMLElement {
  return screen.getByLabelText(/^Owner$/i);
}

describe("/new owner picker (D-ORG-06)", () => {
  it("lists @self + Owner/Admin orgs — not Member-only orgs", async () => {
    const mod = await loadNewModule();
    const NewPage = (mod.NewPage ?? mod.default) as unknown;
    expect(NewPage, "NewPage must export for owner picker").toBeTruthy();
    render(NewPage as never);

    const ownerControl = ownerTrigger();
    expect(
      ownerControl,
      "Owner Select/combobox listing self + Owner/Admin orgs (D-ORG-06)",
    ).toBeTruthy();

    fireEvent.click(ownerControl);
    await waitFor(() => {
      expect(screen.getByRole("option", { name: "@ada" })).toBeInTheDocument();
      expect(screen.getByRole("option", { name: /Acme/ })).toBeInTheDocument();
      expect(screen.getByRole("option", { name: /Widgets/ })).toBeInTheDocument();
    });
    expect(screen.queryByRole("option", { name: /ReadOnly Co/i })).not.toBeInTheDocument();
  }, 20_000);

  it("removes Organizations come in a later phase copy", async () => {
    const mod = await loadNewModule();
    const NewPage = (mod.NewPage ?? mod.default) as unknown;
    render(NewPage as never);

    expect(screen.queryByText(/Organizations come in a later phase/i)).not.toBeInTheDocument();
  }, 20_000);

  it("repo.create posts selected owner slug", async () => {
    createMock.mockResolvedValue({
      ok: true,
      data: {
        id: "r1",
        owner_id: "u1",
        owner_type: "user",
        owner_username: "ada",
        name: "demo",
        description: "",
        visibility: "public",
        default_branch: "main",
        updated_at: "2026-01-01T00:00:00Z",
        can_admin: true,
        can_write: true,
      },
    });

    const mod = await loadNewModule();
    const NewPage = (mod.NewPage ?? mod.default) as unknown;
    render(NewPage as never);

    // Default selection is self; owner slug must still be posted (D-ORG-06).
    expect(ownerTrigger()).toHaveTextContent("@ada");

    fireEvent.input(screen.getByLabelText(/repository name/i), {
      target: { value: "demo" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create repository/i }));

    await waitFor(() => {
      expect(createMock).toHaveBeenCalled();
    });
    const payload = createMock.mock.calls[0]?.[0] as { owner?: string };
    expect(payload.owner).toBe("ada");
  }, 20_000);
});
