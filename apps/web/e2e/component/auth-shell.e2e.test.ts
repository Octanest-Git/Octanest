import { createElement, createRoot, type ComponentBody, type Root } from "octane";
import { afterEach, describe, expect, it } from "vitest";
import { page } from "vitest/browser";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";

let root: Root | null = null;
let host: HTMLElement | null = null;

afterEach(() => {
  root?.unmount();
  host?.remove();
  root = null;
  host = null;
});

function mount(Component: ComponentBody, props?: Record<string, unknown>) {
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
  root.render(Component, props);
}

describe("auth shell (browser e2e)", () => {
  it("shows Octanest brand mark and primary CTA", async () => {
    mount(AuthShell, {
      title: "Sign in to Octanest",
      support: "Welcome back.",
      children: createElement(Button, { type: "button" }, "Sign in"),
    });

    await expect
      .element(page.getByRole("heading", { name: "Sign in to Octanest" }))
      .toBeVisible();
    await expect.element(page.getByAltText("Octanest")).toBeVisible();
    await expect
      .element(page.getByRole("button", { name: "Sign in" }))
      .toBeEnabled();
  });

  it("surfaces auth errors as alerts", async () => {
    mount(AuthErrorBanner, {
      message: "Incorrect email/username or password.",
    });

    await expect
      .element(page.getByRole("alert"))
      .toHaveTextContent("Incorrect email/username or password.");
  });

  it("clicks the primary button in a real browser", async () => {
    let clicked = false;
    mount(Button, {
      type: "button",
      onClick: () => {
        clicked = true;
      },
      children: "Continue",
    });

    await page.getByRole("button", { name: "Continue" }).click();
    expect(clicked).toBe(true);
  });
});
