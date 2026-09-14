import { createElement } from "octane";
import { QueryClient, QueryClientProvider } from "@octanejs/tanstack-query";
import { render } from "@octanejs/testing-library";

/** Integration harness — app root mounts QueryClientProvider; tests must too. */
export function renderWithQueryClient(
  Component: unknown,
  options?: { props?: Record<string, unknown> },
) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  function Harness() {
    return createElement(
      QueryClientProvider as never,
      { client } as never,
      createElement(Component as never, (options?.props ?? {}) as never),
    );
  }
  return render(Harness as never);
}
