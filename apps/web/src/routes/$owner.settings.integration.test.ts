/**
 * Org settings layout + General / Members / Labels render coverage.
 */
import { createElement } from "octane";
import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "@octanest/web/test-runner";
import { vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

const updateSettingsMock = vi.fn();
const membersListMock = vi.fn();
const invitesListMock = vi.fn();
const labelsListMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    org: {
      updateSettings: (...args: unknown[]) => updateSettingsMock(...args),
      members: {
        list: (...args: unknown[]) => membersListMock(...args),
        add: vi.fn(),
        updateRole: vi.fn(),
        remove: vi.fn(),
      },
      invites: {
        list: (...args: unknown[]) => invitesListMock(...args),
        create: vi.fn(),
        revoke: vi.fn(),
      },
    },
    label: {
      listForOrg: (...args: unknown[]) => labelsListMock(...args),
      create: vi.fn(),
      delete: vi.fn(),
    },
    user: {
      lookup: vi.fn().mockResolvedValue({ ok: true, data: { users: [] } }),
    },
  },
}));

const org = {
  id: "o1",
  slug: "acme",
  display_name: "Acme",
  member_base_permission: "none" as const,
  created_at: "2026-01-01T00:00:00Z",
};

let settingsLoader: { org: typeof org; canAdmin: boolean } | undefined = {
  org,
  canAdmin: true,
};
let membersLoader: { org: typeof org; canAdmin: boolean } | undefined = {
  org,
  canAdmin: true,
};
let labelsLoader: { org: typeof org; canAdmin: boolean } | undefined = {
  org,
  canAdmin: true,
};
let pathname = "/acme/settings";

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@octanejs/tanstack-router")>();
  return {
    ...actual,
    Outlet: () => createElement("div", { "data-testid": "org-settings-outlet" }, "outlet"),
    useParams: () => ({ owner: "acme" }),
    useLocation: (opts?: { select?: (loc: { pathname: string }) => unknown }) => {
      const loc = { pathname };
      return opts?.select ? opts.select(loc) : loc;
    },
    useLoaderData: (opts?: { from?: string }) => {
      const from = opts?.from ?? "";
      if (from.includes("members")) return membersLoader;
      if (from.includes("labels")) return labelsLoader;
      return settingsLoader;
    },
  };
});

import { OrgSettingsLayoutPage } from "./$owner.settings";
import { OrgSettingsPage } from "./$owner.settings.index";
import { OrgMembersPage } from "./$owner.settings.members";
import { OrgLabelsPage } from "./$owner.settings.labels";

beforeEach(() => {
  pathname = "/acme/settings";
  settingsLoader = { org, canAdmin: true };
  membersLoader = { org, canAdmin: true };
  labelsLoader = { org, canAdmin: true };
  membersListMock.mockResolvedValue({
    ok: true,
    data: {
      members: [
        {
          user_id: "u1",
          username: "owner1",
          role: "owner",
          created_at: "2026-01-01T00:00:00Z",
        },
      ],
    },
  });
  invitesListMock.mockResolvedValue({ ok: true, data: { invites: [] } });
  labelsListMock.mockResolvedValue({
    ok: true,
    data: {
      labels: [{ id: "l1", name: "bug", color: "d73a4a", description: "Something broken" }],
    },
  });
  updateSettingsMock.mockReset();
});

afterEach(cleanup);

describe("org settings layout", () => {
  it("happy: sidebar links for General, Members, Labels", async () => {
    renderWithQueryClient(OrgSettingsLayoutPage);

    await waitFor(() => {
      expect(screen.getByTestId("org-settings-layout")).toBeInTheDocument();
    });
    const nav = screen.getByRole("navigation", { name: "Organization settings" });
    expect(nav.querySelector('a[href="/acme/settings"]')).toBeTruthy();
    expect(nav.querySelector('a[href="/acme/settings/members"]')).toBeTruthy();
    expect(nav.querySelector('a[href="/acme/settings/labels"]')).toBeTruthy();
    expect(screen.getByTestId("org-settings-outlet")).toBeInTheDocument();
  });

  it("edge: highlights Members when pathname ends with /members", async () => {
    pathname = "/acme/settings/members";
    renderWithQueryClient(OrgSettingsLayoutPage);

    await waitFor(() => {
      expect(screen.getByRole("link", { name: "Members" })).toHaveAttribute("aria-current", "page");
    });
  });

  it("unhappy: loading state when org missing", async () => {
    settingsLoader = undefined;
    renderWithQueryClient(OrgSettingsLayoutPage);
    expect(screen.getByText("Loading…")).toBeInTheDocument();
  });
});

describe("org settings general", () => {
  it("happy: display name + member_base select", async () => {
    renderWithQueryClient(OrgSettingsPage);

    await waitFor(() => {
      expect(screen.getByTestId("org-settings-general")).toBeInTheDocument();
    });
    expect(screen.getByLabelText("Display name")).toHaveValue("Acme");
    expect(screen.getByLabelText("Base permission for Members")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save settings" })).toBeInTheDocument();
  });
});

describe("org settings members", () => {
  it("happy: members table + add member controls", async () => {
    renderWithQueryClient(OrgMembersPage);

    await waitFor(() => {
      expect(screen.getByTestId("org-settings-members")).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(screen.getByText("@owner1")).toBeInTheDocument();
    });
    expect(screen.getByRole("heading", { name: "Members" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Add member" })).toBeInTheDocument();
  });

  it("unhappy: shows loading when org missing", () => {
    membersLoader = undefined;
    renderWithQueryClient(OrgMembersPage);
    expect(screen.getByText("Loading…")).toBeInTheDocument();
  });
});

describe("org settings labels", () => {
  it("happy: create form + existing label list", async () => {
    renderWithQueryClient(OrgLabelsPage);

    await waitFor(() => {
      expect(screen.getByTestId("org-settings-labels")).toBeInTheDocument();
    });
    expect(screen.getByRole("heading", { name: "Labels" })).toBeInTheDocument();
    expect(screen.getByLabelText("Name")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Create label" })).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByText("bug")).toBeInTheDocument();
    });
  });

  it("unhappy: empty catalog message", async () => {
    labelsListMock.mockResolvedValue({ ok: true, data: { labels: [] } });
    renderWithQueryClient(OrgLabelsPage);

    await waitFor(() => {
      expect(screen.getByText("No org labels yet")).toBeInTheDocument();
    });
  });
});
