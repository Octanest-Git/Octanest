import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    system: {
      health: vi.fn(),
    },
  },
}));

import { apiClient } from "@/lib/api-client";
import { StatusPage } from "./status";

afterEach(cleanup);

describe("/status Query health", () => {
  it("shows All systems operational when health is ok", async () => {
    vi.mocked(apiClient.system.health).mockResolvedValueOnce({
      ok: true,
      data: { status: "ok", version: "0.1.0", database: "sqlite" },
    } as never);

    renderWithQueryClient(StatusPage);

    await waitFor(() => {
      expect(screen.getByText("All systems operational")).toBeInTheDocument();
    });
    expect(screen.getByText(/version 0\.1\.0 · database sqlite/)).toBeInTheDocument();
  });

  it("shows unreachable when the RPC fails", async () => {
    vi.mocked(apiClient.system.health).mockResolvedValueOnce({
      ok: false,
      error: { code: "rpc.internal", message: "down" },
    } as never);

    renderWithQueryClient(StatusPage);

    await waitFor(() => {
      expect(screen.getByText("Can’t reach the API")).toBeInTheDocument();
    });
  });

  it("shows degraded when status is not ok", async () => {
    vi.mocked(apiClient.system.health).mockResolvedValueOnce({
      ok: true,
      data: { status: "degraded", version: "0.1.0", database: "sqlite" },
    } as never);

    renderWithQueryClient(StatusPage);

    await waitFor(() => {
      expect(screen.getByText("Degraded or failing")).toBeInTheDocument();
    });
  });
});
