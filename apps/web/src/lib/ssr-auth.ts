import { createServerFn } from "@octanejs/tanstack-start";
import { getRequestHeader } from "@octanejs/tanstack-start/server";
import { createClient, type OctanestClient } from "@octanest/api-client";

/** API origin for SSR Cookie-forward RPCs — never the browser origin during SSR. */
function ssrApiOrigin(): string {
  return (
    process.env.OCTANEST_API_ORIGIN?.replace(/\/$/, "") ||
    process.env.OCTANEST_E2E_API_ORIGIN?.replace(/\/$/, "") ||
    "http://127.0.0.1:8080"
  );
}

/**
 * Cookie-forward Octanest RPC client for server fns / SSR loaders.
 * Forwards the incoming request Cookie only — never logs cookie values (T-06-11).
 */
function createSsrClient(cookie: string): OctanestClient {
  return createClient({
    baseUrl: ssrApiOrigin(),
    credentials: "include",
    fetch: (input, init) => {
      const headers = new Headers(init?.headers);
      if (cookie) {
        headers.set("cookie", cookie);
      }
      return fetch(input, { ...init, headers });
    },
  });
}

function incomingCookie(): string {
  return getRequestHeader("cookie") ?? "";
}

/** SSR: auth.bootstrap_status with Cookie forward. */
export const fetchBootstrapStatus = createServerFn({ method: "GET" }).handler(
  async () => {
    const client = createSsrClient(incomingCookie());
    return client.auth.bootstrapStatus();
  },
);

/** SSR: auth.me with Cookie forward. */
export const fetchSessionMe = createServerFn({ method: "GET" }).handler(
  async () => {
    const client = createSsrClient(incomingCookie());
    return client.auth.me();
  },
);

/** SSR: auth.provider_config (allow_signup) with Cookie forward. */
export const fetchProviderConfig = createServerFn({ method: "GET" }).handler(
  async () => {
    const client = createSsrClient(incomingCookie());
    return client.auth.providerConfig();
  },
);

export type AppAccessRedirectInput = {
  pathname: string;
  needsSetup: boolean;
  mustChangeCredentials: boolean;
};

/**
 * Pure app-access gate (D-09/D-10 + UI-SPEC must_change).
 * needs_setup wins over must_change. Returns redirect path or null (no redirect).
 * Carve-outs: /status (always); /setup while needs_setup; /setup/credentials while must_change.
 */
export function resolveAppAccessRedirect(
  input: AppAccessRedirectInput,
): string | null {
  const pathname = normalizePathname(input.pathname);

  if (input.needsSetup) {
    if (pathname === "/status" || pathname === "/setup") {
      return null;
    }
    return "/setup";
  }

  if (input.mustChangeCredentials) {
    if (pathname === "/status" || pathname === "/setup/credentials") {
      return null;
    }
    return "/setup/credentials";
  }

  return null;
}

function normalizePathname(pathname: string): string {
  if (!pathname) return "/";
  const noQuery = pathname.split("?")[0] ?? "/";
  if (noQuery.length > 1 && noQuery.endsWith("/")) {
    return noQuery.slice(0, -1);
  }
  return noQuery || "/";
}
