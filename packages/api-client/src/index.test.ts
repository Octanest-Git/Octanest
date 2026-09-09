import { describe, expect, it } from "vitest";
import { RPC_VERSION, RPC_VERSION_HEADER, createClient } from "./index";

describe("api-client", () => {
  it("exposes protocol version 1", () => {
    expect(RPC_VERSION).toBe(1);
    expect(RPC_VERSION_HEADER).toBe("Octanest-RPC-Version");
  });

  it("sends version header on health", async () => {
    const calls: RequestInit[] = [];
    const client = createClient({
      baseUrl: "http://example.test",
      fetch: async (_url, init) => {
        calls.push(init ?? {});
        return new Response(
          JSON.stringify({
            ok: true,
            data: { status: "ok", version: "0.1.0", database: "skipped" },
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        );
      },
    });
    const res = await client.system.health();
    expect(res.ok).toBe(true);
    const headers = new Headers(calls[0]?.headers);
    expect(headers.get("Octanest-RPC-Version")).toBe("1");
  });
});
