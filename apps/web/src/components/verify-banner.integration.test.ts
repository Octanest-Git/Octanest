import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

vi.mock("@/lib/toast", () => ({
  toastError: vi.fn(),
  toastSuccess: vi.fn(),
}));

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(),
      resendVerify: vi.fn(),
    },
  },
}));

import { apiClient } from "@/lib/api-client";
import { VerifyBanner } from "./verify-banner";

afterEach(cleanup);

const baseUser = {
  id: "u1",
  email: "u@example.com",
  username: "user",
  display_name: "User",
  bio: "",
  role: "user" as const,
  profile_incomplete: false,
  must_change_credentials: false,
};

describe("VerifyBanner shared auth.me query", () => {
  it("is hidden when signed out", async () => {
    vi.mocked(apiClient.auth.me).mockResolvedValueOnce({
      ok: false,
      error: { code: "auth.unauthenticated", message: "n" },
    } as never);

    const { container } = renderWithQueryClient(VerifyBanner);
    await waitFor(() => {
      expect(apiClient.auth.me).toHaveBeenCalled();
    });
    expect(container.querySelector("[aria-label='Email verification']")).toBeNull();
  });

  it("is hidden when email is verified", async () => {
    vi.mocked(apiClient.auth.me).mockResolvedValueOnce({
      ok: true,
      data: { ...baseUser, email_verified: true },
    } as never);

    const { container } = renderWithQueryClient(VerifyBanner);
    await waitFor(() => {
      expect(apiClient.auth.me).toHaveBeenCalled();
    });
    expect(container.querySelector("[aria-label='Email verification']")).toBeNull();
  });

  it("shows when signed in and unverified", async () => {
    vi.mocked(apiClient.auth.me).mockResolvedValueOnce({
      ok: true,
      data: { ...baseUser, email_verified: false },
    } as never);

    renderWithQueryClient(VerifyBanner);

    await waitFor(() => {
      expect(
        screen.getByRole("status", { name: /email verification/i }),
      ).toBeInTheDocument();
    });
    expect(screen.getByText("Verify your email")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /enter code/i })).toHaveAttribute(
      "href",
      "/verify",
    );
  });
});
