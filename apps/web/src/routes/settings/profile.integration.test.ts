/**
 * RESEARCH P1 / D-QH-03 — settings/profile export/render contracts (happy-dom).
 */
import { createElement } from "octane";
import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

const getProfileMock = vi.fn();
const meMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: (...args: unknown[]) => meMock(...args),
      logout: vi.fn(),
      logoutAll: vi.fn(),
    },
    user: {
      getProfile: (...args: unknown[]) => getProfileMock(...args),
      updateProfile: vi.fn(),
    },
  },
}));

type LoaderShape =
  | { kind: "unauthenticated" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      user: {
        id: string;
        email: string;
        username: string;
        display_name: string;
        bio: string;
        avatar_url: null;
        role: string;
        profile_incomplete: boolean;
        email_verified: boolean;
        must_change_credentials: boolean;
        default_branch: string;
      };
    };

let loaderData: LoaderShape;

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@octanejs/tanstack-router")>();
  return {
    ...actual,
    useLoaderData: () => loaderData,
    Link: (props: {
      to?: string;
      children?: unknown;
      className?: string;
      "aria-current"?: string;
    }) =>
      createElement(
        "a",
        {
          href: props.to ?? "#",
          className: props.className,
          "aria-current": props["aria-current"],
        },
        props.children as never,
      ),
  };
});

import { ProfilePage, Route } from "./profile";

const readyUser = {
  id: "u1",
  email: "profile@octanest.local",
  username: "profileuser",
  display_name: "Profile User",
  bio: "",
  avatar_url: null as null,
  role: "user",
  profile_incomplete: false,
  email_verified: true,
  must_change_credentials: false,
  default_branch: "main",
};

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("/settings/profile Wave 0 contracts (RESEARCH P1)", () => {
  it("exports Route and ProfilePage", () => {
    expect(Route).toBeTruthy();
    expect(typeof ProfilePage).toBe("function");
  });

  it("renders profile form when loader is ready", async () => {
    loaderData = { kind: "ready", user: readyUser };
    getProfileMock.mockResolvedValue({ ok: true, data: readyUser });
    meMock.mockResolvedValue({ ok: true, data: readyUser });

    renderWithQueryClient(ProfilePage);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Profile" })).toBeInTheDocument();
    });
    expect(screen.getByDisplayValue("profileuser")).toBeInTheDocument();
    // Profile is profile-only — theme / default branch / logout live on General.
    expect(screen.queryByLabelText("Default branch name")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Log out" })).not.toBeInTheDocument();
    expect(screen.queryByRole("listbox", { name: "Theme" })).not.toBeInTheDocument();
  });
});
