/**
 * TemplatePicker: selecting a card must not throw Octane insertBefore races when
 * the parent also updates sibling fields (stack → default .gitignore on /new).
 */
import { cleanup, fireEvent, screen, waitFor, within } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { TemplatePickerSiblingHarness } from "@/components/repo/template-picker.harness";
import { trackDomErrors } from "@/test/dom-errors";
import { renderWithQueryClient } from "@/test/render-with-query";

afterEach(cleanup);

function openStackPicker() {
  fireEvent.click(screen.getByLabelText("Stack / template"));
}

async function chooseFromOpenDialog(name: RegExp | string) {
  const dialog = await waitFor(() => screen.getByRole("dialog"));
  fireEvent.click(within(dialog).getByRole("button", { name }));
  await waitFor(() => {
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
}

describe("TemplatePicker DOM races (issue #18)", () => {
  it("selecting a stack with sibling gitignore update does not throw insertBefore", async () => {
    const tracker = trackDomErrors();
    try {
      renderWithQueryClient(TemplatePickerSiblingHarness);

      openStackPicker();
      await chooseFromOpenDialog(/^Next\.js/);

      await waitFor(() => {
        expect(screen.getByTestId("stack-value").textContent).toBe("nextjs");
        expect(screen.getByTestId("gitignore-value").textContent).toBe("Node");
      });

      expect(tracker.domRaceErrors()).toEqual([]);
      expect(screen.getByLabelText("Stack / template")).toHaveTextContent(/Next\.js/);
      expect(screen.getByLabelText(".gitignore")).toHaveTextContent(/Node/);
    } finally {
      tracker.dispose();
    }
  }, 20_000);

  it("switching templates repeatedly stays free of hierarchy errors", async () => {
    const tracker = trackDomErrors();
    try {
      renderWithQueryClient(TemplatePickerSiblingHarness);

      for (const name of [/^Rust/, /^Go/, /^Acme Node/, /^starter/] as const) {
        openStackPicker();
        await chooseFromOpenDialog(name);
      }

      expect(tracker.domRaceErrors()).toEqual([]);
      expect(screen.getByTestId("stack-value").textContent).toBe("repo:r1");
    } finally {
      tracker.dispose();
    }
  }, 30_000);
});
