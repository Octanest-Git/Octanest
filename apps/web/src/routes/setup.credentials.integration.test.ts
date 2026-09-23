import { cleanup, fireEvent, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "@octanest/web/test-runner";
import { vi } from "vitest";

const confirmAdminCredentials = vi.fn();
const me = vi.fn(async () => ({
  ok: true,
  data: {
    id: "u1",
    email: "admin@example.com",
    username: "system-administrator",
    display_name: "system-administrator",
    bio: "",
    role: "sys-admin",
    profile_incomplete: false,
    email_verified: true,
    must_change_credentials: true,
  },
}));

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me,
      confirmAdminCredentials,
    },
  },
}));

vi.mock("@/lib/ssr-auth", () => ({
  fetchSessionMe: vi.fn(async () => ({
    ok: true,
    data: { must_change_credentials: true },
  })),
}));

afterEach(() => {
  cleanup();
  confirmAdminCredentials.mockReset();
  me.mockClear();
});

describe("/setup/credentials (AUTH-06 UI-SPEC)", () => {
  it("exports CredentialsPage with Keep current password Switch and rejects default username", async () => {
    const mod = await import("./setup.credentials");
    expect(mod, "CredentialsPage must be exported for integration tests (06-06)").toHaveProperty(
      "CredentialsPage",
    );

    const { CredentialsPage } = mod as { CredentialsPage: unknown };
    render(CredentialsPage as never);

    await waitFor(() => {
      expect(screen.getByText("Confirm admin account")).toBeInTheDocument();
    });
    expect(screen.getByText("Keep current password")).toBeInTheDocument();

    const username = screen.getByLabelText("Username") as HTMLInputElement;
    expect(username.value.toLowerCase()).toBe("system-administrator");

    fireEvent.submit(username.closest("form")!);

    await waitFor(() => {
      expect(
        screen.getByText(/Choose a username other than the default system-administrator/i),
      ).toBeInTheDocument();
    });
    expect(confirmAdminCredentials).not.toHaveBeenCalled();
  }, 15_000);
});
