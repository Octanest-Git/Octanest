/**
 * MirrorSettingsPanel: HTTPS ↔ SSH auth toggle must not throw Octane
 * insertBefore races (Base UI RadioIndicator mount + sibling panel update).
 *
 * happy-dom does not throw this Chromium HierarchyRequestError — the real gate
 * is stack-browser `expectMirrorAuthToggleFlow`. This suite is a fast smoke for
 * class toggles + CSS-hidden panels. Product fix: RadioGroupItem Indicator
 * `keepMounted` (see `components/ui/radio-group.tsrx`).
 */
import { cleanup, fireEvent, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MirrorSettingsPanel } from "@/components/repo/mirror-settings-panel";
import { trackDomErrors } from "@/test/dom-errors";
import { renderWithQueryClient } from "@/test/render-with-query";

const mirrorGetMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    repo: {
      mirror: {
        get: (...args: unknown[]) => mirrorGetMock(...args),
        upsert: vi.fn(),
        delete: vi.fn(),
        syncNow: vi.fn(),
        generateSshKey: vi.fn(),
        fetchHostKey: vi.fn(),
        rotateWebhookSecret: vi.fn(),
      },
    },
  },
}));

vi.mock("@/lib/toast", () => ({
  toastSuccess: vi.fn(),
}));

beforeEach(() => {
  mirrorGetMock.mockReset();
  mirrorGetMock.mockResolvedValue({ ok: true, data: { mirror: null } });
});

afterEach(cleanup);

describe("MirrorSettingsPanel DOM races", () => {
  it("clicking SSH deploy key does not throw insertBefore", async () => {
    const tracker = trackDomErrors();
    try {
      renderWithQueryClient(MirrorSettingsPanel, {
        props: { owner: "ada", name: "hello", can_admin: true },
      });

      await waitFor(() => {
        expect(screen.getByTestId("repo-mirror-settings")).toBeInTheDocument();
      });

      expect(screen.getByTestId("mirror-auth-https")).not.toHaveClass("hidden");
      expect(screen.getByTestId("mirror-auth-ssh")).toHaveClass("hidden");

      expect(screen.getByTestId("mirror-sync-mode")).toBeInTheDocument();
      fireEvent.click(screen.getByTestId("mirror-sync-mode-exact"));
      await waitFor(() => {
        expect(screen.getByTestId("mirror-exact-warning")).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId("mirror-auth-kind-ssh"));

      await waitFor(() => {
        expect(screen.getByTestId("mirror-auth-ssh")).not.toHaveClass("hidden");
        expect(screen.getByTestId("mirror-auth-https")).toHaveClass("hidden");
      });

      expect(screen.getByLabelText(/Host key/i)).toBeInTheDocument();
      tracker.expectNoDomRaces();
      expect(screen.queryByText(/Something went wrong/i)).not.toBeInTheDocument();
    } finally {
      tracker.dispose();
    }
  }, 20_000);

  it("toggling HTTPS ↔ SSH repeatedly stays free of hierarchy errors", async () => {
    const tracker = trackDomErrors();
    try {
      renderWithQueryClient(MirrorSettingsPanel, {
        props: { owner: "ada", name: "hello", can_admin: true },
      });

      await waitFor(() => {
        expect(screen.getByTestId("repo-mirror-settings")).toBeInTheDocument();
      });

      for (let i = 0; i < 4; i++) {
        fireEvent.click(screen.getByTestId("mirror-auth-kind-ssh"));
        await waitFor(() => {
          expect(screen.getByTestId("mirror-auth-ssh")).not.toHaveClass("hidden");
        });
        fireEvent.click(screen.getByTestId("mirror-auth-kind-https"));
        await waitFor(() => {
          expect(screen.getByTestId("mirror-auth-https")).not.toHaveClass("hidden");
        });
      }

      tracker.expectNoDomRaces();
      expect(screen.queryByText(/Something went wrong/i)).not.toBeInTheDocument();
    } finally {
      tracker.dispose();
    }
  }, 30_000);
});
