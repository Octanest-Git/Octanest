import { describe, expect, it } from "vitest";
import { page } from "vitest/browser";
import { render } from "vitest-browser-react";
import { AuthErrorBanner, AuthShell } from "@/components/auth-shell";
import { Button } from "@/components/ui/button";

describe("auth shell (browser e2e)", () => {
  it("shows Octanest brand mark and primary CTA", async () => {
    await render(
      <AuthShell title="Sign in to Octanest" support="Welcome back.">
        <Button type="button">Sign in</Button>
      </AuthShell>,
    );

    await expect
      .element(page.getByRole("heading", { name: "Sign in to Octanest" }))
      .toBeVisible();
    await expect.element(page.getByAltText("Octanest")).toBeVisible();
    await expect
      .element(page.getByRole("button", { name: "Sign in" }))
      .toBeEnabled();
  });

  it("surfaces auth errors as alerts", async () => {
    await render(
      <AuthErrorBanner message="Incorrect email/username or password." />,
    );

    await expect
      .element(page.getByRole("alert"))
      .toHaveTextContent("Incorrect email/username or password.");
  });

  it("clicks the primary button in a real browser", async () => {
    let clicked = false;
    await render(
      <Button
        type="button"
        onClick={() => {
          clicked = true;
        }}
      >
        Continue
      </Button>,
    );

    await page.getByRole("button", { name: "Continue" }).click();
    expect(clicked).toBe(true);
  });
});
