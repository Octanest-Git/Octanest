/**
 * TopicChipsInput: GitHub-style chips editor — freeform text commits on
 * comma / Enter / autocomplete pick; Backspace pops the last chip.
 * Suggestions come from `repo.topicsSuggest` (mocked).
 */
import { createElement, useState } from "octane";
import { cleanup, fireEvent, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { normalizeTopicSlug, TopicChipsInput } from "@/components/repo/topics-input";
import { renderWithQueryClient } from "@/test/render-with-query";

const topicsSuggestMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    repo: {
      topicsSuggest: (...args: unknown[]) => topicsSuggestMock(...args),
    },
  },
}));

beforeEach(() => {
  topicsSuggestMock.mockReset();
  topicsSuggestMock.mockResolvedValue({ ok: true, data: { topics: [] } });
});

afterEach(cleanup);

function Harness({ initial = [] as string[] }) {
  const [topics, setTopics] = useState<string[]>(initial);
  return createElement(
    TopicChipsInput as never,
    {
      id: "topics-test",
      topics,
      onTopicsChange: setTopics,
    } as never,
  );
}

function input(): HTMLInputElement {
  return screen.getByRole("combobox") as HTMLInputElement;
}

function typeAndCommit(value: string, key = "Enter") {
  fireEvent.input(input(), { target: { value } });
  fireEvent.keyDown(input(), { key });
}

describe("TopicChipsInput", () => {
  it("commits freeform text on Enter and comma, normalized to slugs", async () => {
    renderWithQueryClient(Harness as never);

    typeAndCommit("Rust Lang");
    await waitFor(() => expect(screen.getByText("rust-lang")).toBeInTheDocument());

    typeAndCommit("CLI", ",");
    await waitFor(() => expect(screen.getByText("cli")).toBeInTheDocument());
    expect(input().value).toBe("");
  });

  it("splits pasted comma-separated input into chips", async () => {
    renderWithQueryClient(Harness as never);

    fireEvent.input(input(), { target: { value: "rust, forge ,ci" } });
    await waitFor(() => expect(screen.getByText("rust")).toBeInTheDocument());
    expect(screen.getByText("forge")).toBeInTheDocument();
    // Tail without a trailing comma stays as pending text in the field.
    expect(input().value).toBe("ci");
  });

  it("removes the last chip on Backspace with empty text and via chip ×", async () => {
    renderWithQueryClient(Harness as never, {
      props: { initial: ["rust", "cli"] },
    });

    fireEvent.keyDown(input(), { key: "Backspace" });
    await waitFor(() => expect(screen.queryByText("cli")).not.toBeInTheDocument());
    expect(screen.getByText("rust")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Remove topic rust" }));
    await waitFor(() => expect(screen.queryByText("rust")).not.toBeInTheDocument());
  });

  it("does not commit duplicates or empty segments", async () => {
    renderWithQueryClient(Harness as never, { props: { initial: ["rust"] } });

    typeAndCommit("rust");
    typeAndCommit(",,,");
    await waitFor(() => {
      expect(screen.getAllByText("rust")).toHaveLength(1);
    });
  });

  it("shows suggestions, picks on ArrowDown+Enter and click", async () => {
    topicsSuggestMock.mockResolvedValue({
      ok: true,
      data: {
        topics: [
          { name: "rustfmt", repo_count: 12 },
          { name: "rustlings", repo_count: 3 },
        ],
      },
    });
    renderWithQueryClient(Harness as never);

    fireEvent.focus(input());
    fireEvent.input(input(), { target: { value: "rust" } });

    await waitFor(() => expect(screen.getByTestId("topics-test-listbox")).toBeInTheDocument());
    expect(screen.getByText("rustfmt")).toBeInTheDocument();
    expect(screen.getByText("12 repos")).toBeInTheDocument();

    // ArrowDown highlights first suggestion; Enter commits it as a chip.
    fireEvent.keyDown(input(), { key: "ArrowDown" });
    fireEvent.keyDown(input(), { key: "Enter" });
    await waitFor(() => expect(screen.getByText("rustfmt")).toBeInTheDocument());
    expect(input().value).toBe("");

    // Click a suggestion directly.
    fireEvent.input(input(), { target: { value: "rustl" } });
    await waitFor(() => expect(screen.getByText("rustlings")).toBeInTheDocument());
    fireEvent.mouseDown(screen.getByText("rustlings"));
    await waitFor(() => {
      const chips = screen.getByTestId("topics-test-chips");
      expect(chips.textContent).toContain("rustlings");
    });
  });

  it("hides the input at the 20-topic cap", async () => {
    const full = Array.from({ length: 20 }, (_, i) => `topic-${i}`);
    renderWithQueryClient(Harness as never, { props: { initial: full } });
    expect(screen.queryByRole("combobox")).not.toBeInTheDocument();
  });

  it("normalizeTopicSlug matches server slug rules", () => {
    expect(normalizeTopicSlug("  Hello World ")).toBe("hello-world");
    expect(normalizeTopicSlug("a__b")).toBe("a-b");
    expect(normalizeTopicSlug("-x-")).toBe("x");
    expect(normalizeTopicSlug("!!!")).toBe("");
    expect(normalizeTopicSlug("UPPER")).toBe("upper");
  });
});
