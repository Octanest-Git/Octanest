import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CloneBox } from "./clone-box";

/**
 * GIT-02 Wave 0 stubs: CloneBox HTTPS PAT how-to panel (D-13 / UI-SPEC).
 *
 * RED until PatHowTo embeds in CloneBox (08-12).
 * Do not implement production how-to here.
 */

afterEach(cleanup);

beforeEach(() => {
  Object.defineProperty(window, "location", {
    configurable: true,
    value: {
      ...window.location,
      origin: "http://127.0.0.1:3000",
      assign: vi.fn(),
    },
  });
});

async function openCloneMenu() {
  fireEvent.click(screen.getByRole("button", { name: "Clone or download" }));
  await waitFor(() => {
    expect(screen.getByText("Clone with HTTPS")).toBeInTheDocument();
  });
}

describe("CloneBox PAT how-to Wave 0 (GIT-02 / D-13)", () => {
  it(
    "shows Authenticate with a personal access token how-to with username aliases",
    async () => {
      render(CloneBox, {
        props: {
          owner: "ada",
          repo: "hello",
          refName: "main",
          empty: false,
          publicOrigin: "http://127.0.0.1:3000",
        },
      });

      await openCloneMenu();

      expect(
        screen.getByText("Authenticate with a personal access token"),
      ).toBeInTheDocument();
      expect(
        screen.getByText(
          /Username: your Octanest username, or `git`, `token`, or `oauth2`\./,
        ),
      ).toBeInTheDocument();
      expect(
        screen.getByText(
          /Password: a personal access token — not your account password\./,
        ),
      ).toBeInTheDocument();
    },
    20000,
  );

  it(
    "Create a personal access token CTA points to /settings/tokens",
    async () => {
      render(CloneBox, {
        props: {
          owner: "ada",
          repo: "hello",
          refName: "main",
          empty: false,
          publicOrigin: "http://127.0.0.1:3000",
        },
      });

      await openCloneMenu();

      const cta = screen.getByRole("link", {
        name: "Create a personal access token",
      });
      expect(cta).toHaveAttribute("href", "/settings/tokens");
    },
    20000,
  );
});
