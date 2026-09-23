import { cleanup, render, screen } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "@octanest/web/test-runner";
import { resolveStatusPhase, StatusResolved } from "./status";

afterEach(cleanup);

describe("/status SSR health phases", () => {
  it("shows All systems operational when health is ok", () => {
    const phase = resolveStatusPhase({
      ok: true,
      data: { status: "ok", version: "0.1.0", database: "sqlite" },
    });
    render(StatusResolved, { props: { phase } });

    expect(screen.getByText("All systems operational")).toBeInTheDocument();
    expect(screen.getByText(/version 0\.1\.0 · database sqlite/)).toBeInTheDocument();
  });

  it("shows unreachable when the RPC fails", () => {
    const phase = resolveStatusPhase({
      ok: false,
      error: { code: "rpc.internal", message: "down" },
    });
    render(StatusResolved, { props: { phase } });

    expect(screen.getByText("Can’t reach the API")).toBeInTheDocument();
  });

  it("shows degraded when status is not ok", () => {
    const phase = resolveStatusPhase({
      ok: true,
      data: { status: "degraded", version: "0.1.0", database: "sqlite" },
    });
    render(StatusResolved, { props: { phase } });

    expect(screen.getByText("Degraded or failing")).toBeInTheDocument();
  });
});
