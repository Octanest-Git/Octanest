/**
 * RESEARCH P1 / D-QH-03 — verify route export/render contracts (happy-dom).
 */
import { cleanup, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const meMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: (...args: unknown[]) => meMock(...args),
      verify: vi.fn(),
      requestVerify: vi.fn(),
      resendVerify: vi.fn(),
    },
  },
}));

import { Route, VerifyPage } from "./verify";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  window.history.replaceState({}, "", "/verify");
});

beforeEach(() => {
  window.history.replaceState({}, "", "/verify");
  meMock.mockResolvedValue({
    ok: false,
    error: { code: "auth.unauthenticated", message: "n" },
  });
});

describe("/verify Wave 0 contracts (RESEARCH P1)", () => {
  it("exports Route and VerifyPage", () => {
    expect(Route).toBeTruthy();
    expect(typeof VerifyPage).toBe("function");
  });

  it("prompts sign-in when anonymous", async () => {
    render(VerifyPage);

    await waitFor(() => {
      expect(
        screen.getByText(/Sign in to finish verifying this email/i),
      ).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Sign in" })).toBeInTheDocument();
  });
});
