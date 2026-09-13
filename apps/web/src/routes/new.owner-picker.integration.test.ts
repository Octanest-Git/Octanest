import {
  cleanup,
  render,
  screen,
} from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

/**
 * D-ORG-06 Wave 0 stubs: /new owner picker (self + Owner/Admin orgs).
 *
 * RED until /new owner Select lands (10-11).
 * Do not implement production owner picker here.
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
};

let loaderData: LoaderShape;

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("@octanejs/tanstack-router")>();
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
  };
});

afterEach(cleanup);

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

describe("/new owner picker Wave 0 (D-ORG-06)", () => {
  it("lists @self + Owner/Admin orgs — not Member-only orgs", async () => {
    const mod = await loadNewModule();
    const NewPage = (mod.NewPage ?? mod.default) as unknown;
    expect(NewPage, "Wave 0: NewPage must export for owner picker").toBeTruthy();
    render(NewPage as never);

    // Greened in 10-11: Owner Select/combobox — not locked plain text @ada
    const ownerControl =
      screen.queryByRole("combobox", { name: /owner/i }) ??
      screen.queryByLabelText(/^Owner$/i);
    expect(
      ownerControl,
      "Wave 0: Owner Select/combobox listing self + Owner/Admin orgs (D-ORG-06)",
    ).toBeTruthy();
  });

  it("removes Organizations come in a later phase copy", async () => {
    const mod = await loadNewModule();
    const NewPage = (mod.NewPage ?? mod.default) as unknown;
    render(NewPage as never);

    // Greened in 10-11: placeholder copy gone
    expect(
      screen.queryByText(/Organizations come in a later phase/i),
    ).not.toBeInTheDocument();
  });

  it("repo.create posts selected owner slug", async () => {
    const mod = await loadNewModule();
    const NewPage = (mod.NewPage ?? mod.default) as unknown;
    render(NewPage as never);

    // Greened in 10-11: Owner control present so create can post owner slug
    const ownerControl =
      screen.queryByRole("combobox", { name: /owner/i }) ??
      screen.queryByLabelText(/^Owner$/i);
    expect(
      ownerControl,
      "Wave 0: owner control required before create posts owner slug (D-ORG-06)",
    ).toBeTruthy();
  });
});
