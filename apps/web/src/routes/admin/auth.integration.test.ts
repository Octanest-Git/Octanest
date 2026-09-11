import { cleanup, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(async () => ({
        ok: true,
        data: {
          id: "u1",
          email: "admin@example.com",
          username: "admin",
          role: "sys-admin",
          email_verified: true,
          must_change_credentials: false,
        },
      })),
    },
    admin: {
      auth: {
        getSettings: vi.fn(async () => ({
          ok: true,
          data: {
            provider_mode: "local",
            email_provider: "log",
            from_address: null,
            workos_client_id: null,
            oidc_issuer: null,
            oidc_client_id: null,
            smtp_configured: false,
            resend_configured: false,
            workos_api_key_configured: false,
            oidc_client_secret_configured: false,
            allow_signup: false,
          },
        })),
        updateSettings: vi.fn(),
      },
    },
  },
}));

import { AdminAuthPage } from "./auth";

afterEach(cleanup);

describe("/admin/auth Allow open signup (D-08)", () => {
  it("shows Allow open signup Switch with wizard helper copy", async () => {
    render(AdminAuthPage);

    await waitFor(
      () => {
        expect(screen.getByText("Auth settings")).toBeInTheDocument();
      },
      { timeout: 3000 },
    );

    expect(screen.getByText("Allow open signup")).toBeInTheDocument();
    expect(
      screen.getByText(
        /When off, new accounts can’t self-register/i,
      ),
    ).toBeInTheDocument();
  }, 10000);
});
