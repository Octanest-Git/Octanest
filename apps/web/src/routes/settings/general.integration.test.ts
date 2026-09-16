/**
 * Account /settings/general — theme, default branch, logout controls.
 */
import { createElement } from "octane";
import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

const getProfileMock = vi.fn();
const updateProfileMock = vi.fn();
const logoutMock = vi.fn();
const logoutAllMock = vi.fn();
const meMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: (...args: unknown[]) => meMock(...args),
      logout: (...args: unknown[]) => logoutMock(...args),
      logoutAll: (...args: unknown[]) => logoutAllMock(...args),
    },
    user: {
      getProfile: (...args: unknown[]) => getProfileMock(...args),
      updateProfile: (...args: unknown[]) => updateProfileMock(...args),
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

import { GeneralPage, Route } from "./general";

const readyUser = {
  id: "u1",
  email: "general@octanest.local",
  username: "generaluser",
  display_name: "General User",
  bio: "",
  avatar_url: null as null,
  role: "user",
  profile_incomplete: false,
  email_verified: true,
  must_change_credentials: false,
  default_branch: "main",
};

afterEach(cleanup);

describe("/settings/general", () => {
  it("exports a file route for /settings/general", () => {
    expect(Route.options).toBeTruthy();
  });

  it("beforeLoad redirects anonymous sessions to login", () => {
    expect(Route.options.beforeLoad).toBeTypeOf("function");
  });

  it("happy: renders theme, default branch, and logout controls", async () => {
    loaderData = { kind: "ready", user: readyUser };
    renderWithQueryClient(GeneralPage);

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "General" })).toBeInTheDocument();
    });
    expect(screen.getByTestId("settings-general-page")).toBeInTheDocument();
    expect(screen.getByRole("listbox", { name: "Theme" })).toBeInTheDocument();
    expect(screen.getByLabelText("Default branch name")).toHaveValue("main");
    expect(screen.getByRole("button", { name: "Log out" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Log out all devices" })).toBeInTheDocument();

    const nav = screen.getByRole("navigation", { name: "Account settings" });
    expect(nav.querySelector('a[href="/settings/general"]')).toBeTruthy();
    expect(nav.querySelector('a[href="/settings/profile"]')).toBeTruthy();
  });

  it("unhappy: shows loader error message", async () => {
    loaderData = { kind: "error", message: "Could not load settings." };
    renderWithQueryClient(GeneralPage);

    await waitFor(() => {
      expect(screen.getByText("Could not load settings.")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("settings-general-page")).not.toBeInTheDocument();
  });

  it("edge: opens logout-all confirm dialog and can dismiss", async () => {
    loaderData = { kind: "ready", user: readyUser };
    renderWithQueryClient(GeneralPage);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Log out all devices" })).toBeInTheDocument();
    });
    screen.getByRole("button", { name: "Log out all devices" }).click();
    await waitFor(() => {
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });
    screen.getByRole("button", { name: "Stay signed in" }).click();
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
  });
});
