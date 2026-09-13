import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CloneBox } from "./clone-box";

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

describe("CloneBox (E12 / D-22 / D-29)", () => {
  it("shows HTTPS clone URL, SSH placeholder, and enabled archive items when not empty", async () => {
    render(CloneBox, {
      props: {
        owner: "ada",
        repo: "hello",
        refName: "main",
        empty: false,
      },
    });

    await openCloneMenu();

    const httpsUrl = "http://127.0.0.1:3000/ada/hello.git";
    const code = screen.getByTitle(httpsUrl);
    expect(code).toHaveTextContent(httpsUrl);
    expect(code.className).toMatch(/truncate/);

    const copyBtn = screen.getByRole("button", { name: "Copy HTTPS URL" });
    expect(copyBtn).toBeInTheDocument();
    expect(copyBtn.querySelector("svg")).not.toBeNull();

    expect(
      screen.getByText("SSH cloning arrives in a later phase."),
    ).toBeInTheDocument();

    const zip = screen.getByRole("menuitem", { name: "Download ZIP" });
    const tar = screen.getByRole("menuitem", { name: "Download tar.gz" });
    expect(zip).not.toHaveAttribute("aria-disabled", "true");
    expect(tar).not.toHaveAttribute("aria-disabled", "true");
    expect(zip).not.toBeDisabled();
    expect(tar).not.toBeDisabled();
  });

  it("disables archive downloads when the repo is empty", async () => {
    render(CloneBox, {
      props: {
        owner: "ada",
        repo: "empty",
        refName: "main",
        empty: true,
      },
    });

    await openCloneMenu();

    expect(
      screen.getByTitle("http://127.0.0.1:3000/ada/empty.git"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("SSH cloning arrives in a later phase."),
    ).toBeInTheDocument();

    const zip = screen.getByRole("menuitem", { name: "Download ZIP" });
    const tar = screen.getByRole("menuitem", { name: "Download tar.gz" });
    expect(zip).toHaveAttribute("aria-disabled", "true");
    expect(tar).toHaveAttribute("aria-disabled", "true");
  });

  it("assigns archive URL for the current ref when Download ZIP is chosen", async () => {
    const assign = vi.fn();
    Object.defineProperty(window, "location", {
      configurable: true,
      value: {
        origin: "http://127.0.0.1:3000",
        assign,
      },
    });

    render(CloneBox, {
      props: {
        owner: "ada",
        repo: "hello",
        refName: "feature/x",
        empty: false,
      },
    });

    await openCloneMenu();
    fireEvent.click(screen.getByRole("menuitem", { name: "Download ZIP" }));

    expect(assign).toHaveBeenCalledWith(
      "/api/repos/ada/hello/archive/feature%2Fx.zip",
    );
  });
});
