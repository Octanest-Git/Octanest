import { describe, expect, it } from "bun:test";

import { networkErrorMessage, rpcErrorMessage } from "../../src/lib/rpc-error-message.ts";

describe("rpcErrorMessage (bun:test PoC)", () => {
  it("hints migrate/schema for generic auth.internal", () => {
    expect(rpcErrorMessage({ code: "auth.internal", message: "bootstrap operation failed" })).toBe(
      "Database not ready. Migrations may still be running — wait a moment and refresh.",
    );
    expect(rpcErrorMessage({ code: "auth.internal" })).toContain("Database not ready");
  });

  it("preserves specific auth.internal messages", () => {
    expect(
      rpcErrorMessage({ code: "auth.internal", message: "email provider misconfigured" }),
    ).toBe("email provider misconfigured");
  });

  it("prefers non-internal API messages", () => {
    expect(rpcErrorMessage({ code: "auth.setup_required", message: "Complete setup." })).toBe(
      "Complete setup.",
    );
  });

  it("falls back for null/network", () => {
    expect(rpcErrorMessage(null)).toBe(networkErrorMessage());
  });
});
