/**
 * RESEARCH P1 / D-QH-03 — login route export/render contracts (happy-dom).
 */
import { cleanup, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(async () => ({
        ok: false,
        error: { code: "auth.unauthenticated", message: "n" },
      })),
      login: vi.fn(),
      providerConfig: vi.fn(async () => ({
        ok: true,
        data: { mode: "local", allow_signup: true },
      })),
    },
  },
}));

vi.mock("@/lib/ssr-auth", () => ({
  fetchProviderConfig: vi.fn(async () => ({
    ok: true,
    data: { mode: "local", allow_signup: true },
  })),
  fetchSessionMe: vi.fn(async () => ({
    ok: false,
    error: { code: "auth.unauthenticated", message: "n" },
  })),
}));

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("@octanejs/tanstack-router")>();
  return {
    ...actual,
    useLoaderData: () => ({
      mode: "local",
      allow_signup: true,
      loadError: "",
    }),
  };
});

import { LoginPage, Route } from "./login";

afterEach(cleanup);

describe("/login Wave 0 contracts (RESEARCH P1)", () => {
  it("exports Route and LoginPage", () => {
    expect(Route).toBeTruthy();
    expect(typeof LoginPage).toBe("function");
  });

  it("renders local sign-in form fields", async () => {
    render(LoginPage);

    await waitFor(() => {
      expect(screen.getByLabelText("Email or username")).toBeInTheDocument();
    });
    expect(screen.getByLabelText("Password")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Sign in" })).toBeInTheDocument();
    expect(screen.getByText("Create an account")).toBeInTheDocument();
  });
});
