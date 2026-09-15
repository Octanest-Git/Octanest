import type { RepoPublic, UserPublic } from "@octanest/api-client";
import { createElement } from "octane";
import { cleanup, render, screen } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@octanejs/tanstack-router", () => ({
  Link: (props: { to?: string; children?: unknown; className?: string }) =>
    createElement(
      "a",
      { href: props.to ?? "#", className: props.className },
      props.children as never,
    ),
}));

import { SignedInHome } from "./signed-in-home";

afterEach(cleanup);

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
    default_branch: "main",
    ...overrides,
  };
}

function repo(overrides: Partial<RepoPublic> = {}): RepoPublic {
  return {
    id: "r1",
    owner_id: "u1",
    owner_type: "user",
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
  it("unverified: disabled CTA + verify-email hint", () => {
    render(SignedInHome, {
      props: { user: user({ email_verified: false }), repos: [] },
    });

    expect(screen.getByRole("heading", { name: "Your repositories" })).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Create your first repository" }),
    ).toBeInTheDocument();

    const ctas = screen.getAllByRole("button", { name: "New repository" });
    expect(ctas.length).toBeGreaterThanOrEqual(1);
    for (const cta of ctas) {
      expect(cta).toBeDisabled();
      expect(cta).toHaveAttribute("aria-disabled", "true");
      expect(cta).toHaveAttribute("title", "Verify your email to create a repository.");
    }
    expect(screen.getByText("Verify your email to create a repository.")).toBeInTheDocument();
  });

  it("verified: enabled New repository navigates to /new", () => {
    render(SignedInHome, {
      props: { user: user({ email_verified: true }), repos: [] },
    });

    expect(screen.getByRole("heading", { name: "Your repositories" })).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Create your first repository" }),
    ).toBeInTheDocument();

    const ctas = screen.getAllByRole("link", { name: "New repository" });
    expect(ctas.length).toBeGreaterThanOrEqual(1);
    for (const cta of ctas) {
      expect(cta).toHaveAttribute("href", "/new");
      expect(cta).not.toHaveAttribute("aria-disabled", "true");
    }
    expect(screen.queryByText("Verify your email to create a repository.")).not.toBeInTheDocument();
    expect(
      screen.queryByText("Repository creation arrives in a later phase."),
    ).not.toBeInTheDocument();
  });
});

describe("SignedInHome dashboard IA (D-13 / UI E1)", () => {
  it("empty list shows Create your first repository hero and activity placeholder", () => {
    render(SignedInHome, {
      props: { user: user({ email_verified: true }), repos: [] },
    });

    expect(
      screen.getByRole("heading", { name: "Create your first repository" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Your repositories" })).toBeInTheDocument();
    expect(screen.getByText("Activity will show up here.")).toBeInTheDocument();
  });

  it("populated list shows owner/name and visibility Badge", () => {
    render(SignedInHome, {
      props: {
        user: user({ email_verified: true }),
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

    expect(screen.getByText("ada/hello")).toBeInTheDocument();
    expect(screen.getByText("ada/secrets")).toBeInTheDocument();
    expect(screen.getByText("Public")).toBeInTheDocument();
    expect(screen.getByText("Private")).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "Create your first repository" }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Activity will show up here.")).toBeInTheDocument();
  });

  it("keeps incomplete-profile banner above the list", () => {
    render(SignedInHome, {
      props: {
        user: user({ profile_incomplete: true, email_verified: true }),
        repos: [],
      },
    });

    expect(screen.getByRole("status")).toHaveTextContent("Choose a username to finish setup.");
    expect(screen.getByRole("heading", { name: "Your repositories" })).toBeInTheDocument();
  });

  it("shows reposError without empty hero", () => {
    render(SignedInHome, {
      props: {
        user: user({ email_verified: true }),
        repos: [],
        reposError: "Could not load repositories.",
      },
    });

    expect(screen.getByRole("alert")).toHaveTextContent("Could not load repositories.");
    expect(
      screen.queryByRole("heading", { name: "Create your first repository" }),
    ).not.toBeInTheDocument();
  });
});
