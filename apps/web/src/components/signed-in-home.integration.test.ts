import type { RepoPublic, UserPublic } from "@octanest/api-client";
import { createElement } from "octane";
import { cleanup, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@octanejs/tanstack-router", () => ({
  Link: (props: {
    to?: string;
    children?: unknown;
    className?: string;
  }) =>
    createElement(
      "a",
      { href: props.to ?? "#", className: props.className },
      props.children as never,
    ),
}));

const listMineMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    repo: {
      listMine: (...args: unknown[]) => listMineMock(...args),
    },
  },
}));

import { SignedInHome } from "./signed-in-home";

afterEach(cleanup);

beforeEach(() => {
  listMineMock.mockReset();
  listMineMock.mockResolvedValue({ ok: true, data: { repos: [] } });
});

function user(overrides: Partial<UserPublic> = {}): UserPublic {
  return {
    id: "u1",
    email: "ada@example.com",
    username: "ada",
    display_name: "Ada",
    bio: "",
    avatar_url: null,
    role: "user",
    profile_incomplete: false,
    email_verified: false,
    must_change_credentials: false,
    ...overrides,
  };
}

function repo(overrides: Partial<RepoPublic> = {}): RepoPublic {
  return {
    id: "r1",
    owner_id: "u1",
    owner_username: "ada",
    name: "hello",
    description: "A demo repo",
    visibility: "public",
    default_branch: "main",
    updated_at: "2026-09-12T12:00:00Z",
    ...overrides,
  };
}

describe("SignedInHome New repository CTA (D-01 / D-11)", () => {
  it("unverified: disabled CTA + verify-email hint", async () => {
    render(SignedInHome, { props: { user: user({ email_verified: false }) } });

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Your repositories" })).toBeInTheDocument();
    });

    const cta = screen.getByRole("button", { name: "New repository" });
    expect(cta).toBeDisabled();
    expect(cta).toHaveAttribute("aria-disabled", "true");
    expect(cta).toHaveAttribute(
      "title",
      "Verify your email to create a repository.",
    );
    expect(
      screen.getByText("Verify your email to create a repository."),
    ).toBeInTheDocument();
  });

  it("verified: enabled New repository navigates to /new", async () => {
    render(SignedInHome, { props: { user: user({ email_verified: true }) } });

    await waitFor(() => {
      expect(screen.getByRole("heading", { name: "Your repositories" })).toBeInTheDocument();
    });

    const cta = screen.getByRole("link", { name: "New repository" });
    expect(cta).toHaveAttribute("href", "/new");
    expect(cta).not.toHaveAttribute("aria-disabled", "true");
    expect(
      screen.queryByText("Verify your email to create a repository."),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByText("Repository creation arrives in a later phase."),
    ).not.toBeInTheDocument();
  });
});

describe("SignedInHome dashboard IA (D-13 / UI E1)", () => {
  it("empty list shows Create your first repository hero and activity placeholder", async () => {
    listMineMock.mockResolvedValue({ ok: true, data: { repos: [] } });
    render(SignedInHome, { props: { user: user({ email_verified: true }) } });

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Create your first repository" }),
      ).toBeInTheDocument();
    });
    expect(screen.getByRole("heading", { name: "Your repositories" })).toBeInTheDocument();
    expect(screen.getByText("Activity will show up here.")).toBeInTheDocument();
    expect(listMineMock).toHaveBeenCalled();
  });

  it("populated list shows owner/name and visibility Badge", async () => {
    listMineMock.mockResolvedValue({
      ok: true,
      data: {
        repos: [
          repo({ name: "hello", visibility: "public" }),
          repo({
            id: "r2",
            name: "secrets",
            visibility: "private",
            updated_at: "2026-09-11T12:00:00Z",
          }),
        ],
      },
    });
    render(SignedInHome, { props: { user: user({ email_verified: true }) } });

    await waitFor(() => {
      expect(screen.getByText("ada/hello")).toBeInTheDocument();
    });
    expect(screen.getByText("ada/secrets")).toBeInTheDocument();
    expect(screen.getByText("Public")).toBeInTheDocument();
    expect(screen.getByText("Private")).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "Create your first repository" }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Activity will show up here.")).toBeInTheDocument();
  });

  it("keeps incomplete-profile banner above the list", async () => {
    render(SignedInHome, {
      props: { user: user({ profile_incomplete: true, email_verified: true }) },
    });

    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent(
        "Choose a username to finish setup.",
      );
    });
    expect(screen.getByRole("heading", { name: "Your repositories" })).toBeInTheDocument();
  });
});
