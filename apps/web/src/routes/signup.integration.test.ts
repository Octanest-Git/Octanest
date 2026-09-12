import { cleanup, render, screen, waitFor } from "@octanejs/testing-library";
import { isNotFound } from "@octanejs/tanstack-router";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@/lib/bootstrap", () => ({
  redirectIfNeedsSetup: vi.fn(async () => false),
}));

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(async () => ({
        ok: false,
        error: { code: "auth.unauthenticated", message: "n" },
      })),
      providerConfig: vi.fn(async () => ({
        ok: true,
        data: { mode: "local", allow_signup: true },
      })),
      signup: vi.fn(),
    },
  },
}));

vi.mock("@/lib/ssr-auth", () => ({
  fetchProviderConfig: vi.fn(),
  fetchSessionMe: vi.fn(async () => ({
    ok: false,
    error: { code: "auth.unauthenticated", message: "n" },
  })),
}));

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@octanejs/tanstack-router")>();
  return {
    ...actual,
    useLoaderData: () => ({ mode: "local", loadError: "" }),
  };
});

import { fetchProviderConfig } from "@/lib/ssr-auth";
import { Route, SignupPage } from "./signup";

afterEach(cleanup);

describe("/signup closed-signup SSR (D-06)", () => {
  it("beforeLoad calls notFound when allow_signup is false", async () => {
    expect(
      Route.options.beforeLoad,
      "signup beforeLoad must gate closed signup with notFound()",
    ).toBeTypeOf("function");

    vi.mocked(fetchProviderConfig).mockResolvedValue({
      ok: true,
      data: { mode: "local", allow_signup: false },
    });

    let caught: unknown;
    try {
      await Route.options.beforeLoad!({} as never);
    } catch (e) {
      caught = e;
    }
    expect(
      isNotFound(caught),
      "closed signup must throw notFound() — no AuthShell soft page",
    ).toBe(true);
  });

  it("beforeLoad allows render when allow_signup is true", async () => {
    expect(Route.options.beforeLoad).toBeTypeOf("function");

    vi.mocked(fetchProviderConfig).mockResolvedValue({
      ok: true,
      data: { mode: "local", allow_signup: true },
    });

    await expect(
      Route.options.beforeLoad!({} as never),
    ).resolves.toBeUndefined();
  });
});

describe("SignupPage AUTH-05 (no invite)", () => {
  it("local signup form has email/username/password only — no invite fields", async () => {
    render(SignupPage);

    await waitFor(() => {
      expect(screen.getByLabelText("Email")).toBeInTheDocument();
    });
    expect(screen.getByLabelText("Username")).toBeInTheDocument();
    expect(screen.getByLabelText("Password")).toBeInTheDocument();
    expect(screen.getByLabelText("Confirm password")).toBeInTheDocument();

    expect(screen.queryByLabelText(/invite/i)).not.toBeInTheDocument();
    expect(screen.queryByPlaceholderText(/invite/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/invite/i)).not.toBeInTheDocument();
    expect(screen.queryByRole("textbox", { name: /invite/i })).not.toBeInTheDocument();
  });
});
