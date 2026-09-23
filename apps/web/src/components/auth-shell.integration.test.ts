import { createElement } from "octane";
import { cleanup, render, screen } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "@octanest/web/test-runner";
import { AuthErrorBanner, AuthShell } from "./auth-shell";
import { Button } from "./ui/button";

afterEach(cleanup);

describe("AuthShell", () => {
  it("renders brand heading and support copy", () => {
    render(AuthShell, {
      props: {
        title: "Sign in to Octanest",
        support: "Use your account.",
        children: createElement(Button, { type: "button" }, "Continue"),
      },
    });

    expect(screen.getByRole("heading", { name: "Sign in to Octanest" })).toBeInTheDocument();
    expect(screen.getByText("Use your account.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Continue" })).toBeInTheDocument();
  });
});

describe("AuthErrorBanner", () => {
  it("exposes role=alert when message is set", () => {
    render(AuthErrorBanner, { props: { message: "Incorrect password." } });
    expect(screen.getByRole("alert")).toHaveTextContent("Incorrect password.");
  });

  it("renders nothing when message is empty", () => {
    const { container } = render(AuthErrorBanner, { props: { message: "" } });
    expect(container).toBeEmptyDOMElement();
  });
});
