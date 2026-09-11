import { createElement } from "octane";
import { cleanup, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@octanejs/tanstack-router", () => ({
  Link: (props: {
    to?: string;
    children?: unknown;
    className?: string;
    onClick?: () => void;
  }) =>
    createElement(
      "a",
      {
        href: props.to ?? "#",
        className: props.className,
        onClick: props.onClick,
      },
      props.children as never,
    ),
  useNavigate: () => () => undefined,
}));

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(async () => ({
        ok: false,
        error: { code: "auth.unauthenticated", message: "n" },
      })),
      bootstrapStatus: vi.fn(async () => ({
        ok: true,
        data: { needs_setup: false },
      })),
      providerConfig: vi.fn(async () => ({
        ok: true,
        data: { mode: "local", allow_signup: false },
      })),
    },
  },
}));

import { apiClient } from "@/lib/api-client";
import { SiteHeader } from "./chrome";

afterEach(cleanup);

beforeEach(() => {
  vi.mocked(apiClient.auth.me).mockResolvedValue({
    ok: false,
    error: { code: "auth.unauthenticated", message: "n" },
  } as never);
  vi.mocked(apiClient.auth.bootstrapStatus).mockResolvedValue({
    ok: true,
    data: { needs_setup: false },
  } as never);
  vi.mocked(apiClient.auth.providerConfig).mockResolvedValue({
    ok: true,
    data: { mode: "local", allow_signup: false },
  } as never);
});

describe("chrome Wave 0 (D-06 omit Sign up)", () => {
  it("omits Sign up when allow_signup is false", async () => {
    render(SiteHeader);

    await waitFor(() => {
      expect(screen.getByRole("link", { name: /sign in/i })).toBeInTheDocument();
    });

    expect(
      screen.queryByRole("link", { name: /^sign up$/i }),
    ).not.toBeInTheDocument();
  });

  it("omits Sign up when allow_signup is unknown", async () => {
    vi.mocked(apiClient.auth.providerConfig).mockResolvedValueOnce({
      ok: true,
      data: { mode: "local" },
    } as never);

    render(SiteHeader);

    await waitFor(() => {
      expect(screen.getByRole("link", { name: /sign in/i })).toBeInTheDocument();
    });

    expect(
      screen.queryByRole("link", { name: /^sign up$/i }),
    ).not.toBeInTheDocument();
  });

  it("omits Sign in and Sign up while needs_setup", async () => {
    vi.mocked(apiClient.auth.bootstrapStatus).mockResolvedValueOnce({
      ok: true,
      data: { needs_setup: true },
    } as never);

    render(SiteHeader);

    await waitFor(() => {
      expect(screen.queryByLabelText(/^Account$/i)).toBeTruthy();
    });

    expect(
      screen.queryByRole("link", { name: /sign in/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("link", { name: /^sign up$/i }),
    ).not.toBeInTheDocument();
  });

  it("shows Sign up when allow_signup is true", async () => {
    vi.mocked(apiClient.auth.providerConfig).mockResolvedValueOnce({
      ok: true,
      data: { mode: "local", allow_signup: true },
    } as never);

    render(SiteHeader);

    await waitFor(() => {
      expect(screen.getByRole("link", { name: /^sign up$/i })).toBeInTheDocument();
    });

    expect(screen.getByRole("link", { name: /sign in/i })).toBeInTheDocument();
  });
});
