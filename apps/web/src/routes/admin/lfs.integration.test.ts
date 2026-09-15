import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

const meMock = vi.fn();
const getSettingsMock = vi.fn();
const getUsageMock = vi.fn();
const updateSettingsMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: (...args: unknown[]) => meMock(...args),
    },
    admin: {
      lfs: {
        getSettings: (...args: unknown[]) => getSettingsMock(...args),
        getUsage: (...args: unknown[]) => getUsageMock(...args),
        updateSettings: (...args: unknown[]) => updateSettingsMock(...args),
      },
    },
  },
}));

const sysAdmin = {
  id: "u1",
  email: "admin@example.com",
  username: "admin",
  display_name: "Admin",
  bio: "",
  avatar_url: null as null,
  role: "sys-admin" as const,
  profile_incomplete: false,
  email_verified: true,
  must_change_credentials: false,
  default_branch: "main",
};

const readySettings = {
  max_object_bytes: 1024,
  quota_repo_bytes: 2048,
  quota_user_bytes: 4096,
  max_object_bytes_overridden: false,
  quota_repo_bytes_overridden: false,
  quota_user_bytes_overridden: false,
};

const readyUsage = {
  object_count: 0,
  physical_bytes: 0,
  logical_bytes: 0,
  by_repo: [] as unknown[],
  by_owner: [] as unknown[],
};

type LoaderShape =
  | { kind: "unauthenticated" }
  | { kind: "forbidden" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      me: typeof sysAdmin;
      settings: typeof readySettings;
      usage: typeof readyUsage;
    };

let loaderData: LoaderShape | undefined;

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("@octanejs/tanstack-router")>();
  return {
    ...actual,
    useLoaderData: () => loaderData,
  };
});

import { AdminLfsPage } from "./lfs";

/**
 * D-LFS-12 / D-LFS-13 / D-LFS-19 + G-11.1-15: Admin LFS quotas must *render*
 * (raw-source checks alone missed missing useState / Octane template breakage).
 */
describe("admin LFS quotas (D-LFS-12 / D-LFS-13 / D-LFS-19)", () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    meMock.mockReset();
    getSettingsMock.mockReset();
    getUsageMock.mockReset();
    updateSettingsMock.mockReset();
    meMock.mockResolvedValue({ ok: true, data: sysAdmin });
    getSettingsMock.mockResolvedValue({
      ok: true,
      data: readySettings,
    });
    getUsageMock.mockResolvedValue({
      ok: true,
      data: readyUsage,
    });
    loaderData = {
      kind: "ready",
      me: sysAdmin,
      settings: readySettings,
      usage: readyUsage,
    };
  });

  it("exports AdminLfsPage without @else if / bare Loading text and declares error/pending state", async () => {
    const mod = await import("./lfs");
    expect(mod.AdminLfsPage ?? mod.default).toBeTruthy();
    const src = await import("./lfs.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(src).toMatch(/getSettings/);
    expect(src).toMatch(/updateSettings/);
    expect(src).toMatch(/getUsage/);
    expect(src).toMatch(/fetchAdminLfsSettings|loader:/);
    expect(src).toMatch(/AdminLfsSkeleton/);
    expect(src).toMatch(/setError|\[error,/);
    expect(src).toMatch(/setPending|\[pending,/);
    expect(src).not.toMatch(/@else if/);
    expect(src).not.toMatch(/Loading…|Loading usage/);
  });

  it("renders Git LFS quotas form for sys-admin without throwing", async () => {
    renderWithQueryClient(AdminLfsPage);

    await waitFor(
      () => {
        expect(screen.getByTestId("admin-lfs-page")).toBeTruthy();
        expect(screen.getByText("Git LFS quotas")).toBeTruthy();
        expect(screen.getByLabelText(/Max object bytes/i)).toBeTruthy();
        expect(screen.getByText("Instance usage breakdown")).toBeTruthy();
      },
      { timeout: 10_000 },
    );
    expect(screen.queryByText("Loading…")).toBeNull();
    expect(screen.queryByText("Loading usage…")).toBeNull();
  });
});
