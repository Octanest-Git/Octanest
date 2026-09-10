import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AuthErrorBanner, AuthShell } from "./auth-shell";
import { Button } from "./ui/button";

describe("AuthShell", () => {
  it("renders brand heading and support copy", () => {
    render(
      <AuthShell title="Sign in to Octanest" support="Use your account.">
        <Button type="button">Continue</Button>
      </AuthShell>,
    );

    expect(
      screen.getByRole("heading", { name: "Sign in to Octanest" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Use your account.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Continue" })).toBeInTheDocument();
  });
});

describe("AuthErrorBanner", () => {
  it("exposes role=alert when message is set", () => {
    render(<AuthErrorBanner message="Incorrect password." />);
    expect(screen.getByRole("alert")).toHaveTextContent("Incorrect password.");
  });

  it("renders nothing when message is empty", () => {
    const { container } = render(<AuthErrorBanner message="" />);
    expect(container).toBeEmptyDOMElement();
  });
});
