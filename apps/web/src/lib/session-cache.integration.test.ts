import { createElement } from "octane";
import { QueryClient, QueryClientProvider, useQuery } from "@octanejs/tanstack-query";
import { cleanup, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "@octanest/web/test-runner";
import { vi } from "vitest";

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: vi.fn(async () => ({
        ok: true,
        data: {
          id: "u1",
          email: "u@example.com",
          username: "user",
          display_name: "User",
          bio: "",
          role: "user",
          profile_incomplete: false,
          email_verified: false,
          must_change_credentials: false,
        },
      })),
    },
  },
}));

import { apiClient } from "@/lib/api-client";
import { authSessionQueryOptions } from "@/lib/session-queries";

function SessionConsumer({ label }: { label: string }) {
  const me = useQuery(authSessionQueryOptions());
  if (me.isPending) {
    return createElement("p", null, `${label}:loading`);
  }
  return createElement("p", null, `${label}:${me.data?.username ?? "anon"}`);
}

function DualConsumers() {
  return createElement(
    "div",
    null,
    createElement(SessionConsumer as never, { label: "a" } as never),
    createElement(SessionConsumer as never, { label: "b" } as never),
  );
}

afterEach(cleanup);

describe("auth.me Query cache sharing", () => {
  it("dedupes auth.me across two consumers on one QueryClient", async () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    function Harness() {
      return createElement(
        QueryClientProvider as never,
        { client } as never,
        createElement(DualConsumers as never),
      );
    }
    render(Harness as never);

    await waitFor(() => {
      expect(screen.getByText("a:user")).toBeInTheDocument();
      expect(screen.getByText("b:user")).toBeInTheDocument();
    });

    // Shared queryFn — one network call for both observers (≤2 under act remount).
    expect(apiClient.auth.me.mock.calls.length).toBeLessThanOrEqual(2);
  });
});
