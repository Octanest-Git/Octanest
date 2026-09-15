import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(),
    },
    admin: {
      auth: {
        getSettings: vi.fn(),
        updateSettings: vi.fn(),
        factoryReset: vi.fn(),
      },
    },
  },
}));

import { apiClient } from "@/lib/api-client";
import { AdminAuthPage } from "./auth";

afterEach(cleanup);

const sysAdmin = {
  id: "u1",
  email: "admin@example.com",
  username: "admin",
  display_name: "Admin",
  bio: "",
  role: "sys-admin" as const,
  profile_incomplete: false,
  email_verified: true,
  must_change_credentials: false,
};

const settings = {
  provider_mode: "local" as const,
  email_provider: "log" as const,
  from_address: null,
  workos_client_id: null,
  oidc_issuer: null,
  oidc_client_id: null,
  smtp_configured: false,
  resend_configured: false,
  workos_api_key_configured: false,
  oidc_client_secret_configured: false,
  allow_signup: false,
};

describe("/admin/auth Query-backed settings", () => {
  it("shows Allow open signup Switch with wizard helper copy", async () => {
    vi.mocked(apiClient.auth.me).mockResolvedValue({
      ok: true,
      data: sysAdmin,
    } as never);
    vi.mocked(apiClient.admin.auth.getSettings).mockResolvedValue({
      ok: true,
      data: settings,
    } as never);

    renderWithQueryClient(AdminAuthPage);

    await waitFor(
      () => {
        expect(screen.getByText("Auth settings")).toBeInTheDocument();
      },
      { timeout: 3000 },
    );

    expect(screen.getByText("Allow open signup")).toBeInTheDocument();
    expect(screen.getByText(/When off, new accounts can’t self-register/i)).toBeInTheDocument();
    expect(screen.getByText("Danger zone")).toBeInTheDocument();
  }, 10000);

  it("shows forbidden for non sys-admin", async () => {
    vi.mocked(apiClient.auth.me).mockResolvedValue({
      ok: true,
      data: { ...sysAdmin, role: "user" },
    } as never);

    renderWithQueryClient(AdminAuthPage);

    await waitFor(() => {
      expect(
        screen.getByText(/You need admin access to manage auth settings/i),
      ).toBeInTheDocument();
    });
    expect(apiClient.admin.auth.getSettings).not.toHaveBeenCalled();
  });
});
