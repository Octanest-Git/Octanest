import type { UserPublic } from "@octanest/api-client";
import { createElement } from "octane";
import { cleanup, render, screen } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

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
    ...overrides,
  };
}

describe("SignedInHome New repository CTA (D-01 / D-11)", () => {
  it("unverified: disabled CTA + verify-email hint", () => {
    render(SignedInHome, { props: { user: user({ email_verified: false }) } });

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

  it("verified: enabled New repository navigates to /new", () => {
    render(SignedInHome, { props: { user: user({ email_verified: true }) } });

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
