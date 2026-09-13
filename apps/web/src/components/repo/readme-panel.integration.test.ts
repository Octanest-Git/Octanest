import { cleanup, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { ReadmePanel } from "./readme-panel";

afterEach(cleanup);

describe("ReadmePanel sanitize (D-18 / 07-15 UAT)", () => {
  it("renders README HTML without script tags from untrusted markdown", async () => {
    render(ReadmePanel, {
      props: {
        fileName: "README.md",
        markdown:
          '# Hello\n\n<script>alert("xss")</script>\n\nSafe **text** and [link](https://example.com).',
      },
    });

    await waitFor(() => {
      expect(screen.getByRole("heading", { level: 2, name: "README.md" })).toBeInTheDocument();
      expect(screen.getByText(/Safe/)).toBeInTheDocument();
    });

    const section = screen.getByLabelText("README.md");
    expect(section.innerHTML).not.toMatch(/<script/i);
    expect(section.querySelector("script")).toBeNull();
    expect(section.innerHTML).toMatch(/<strong>text<\/strong>/i);
  });
});
