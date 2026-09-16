/**
 * Container / Compose probe for the web process (PLAT smoke, docker healthcheck).
 *
 * Deliberately gated: only `Octanest-Health-Probe: 1` returns 200 JSON.
 * Anything else (including Traefik/public browsers) gets 404 so the endpoint
 * is not a free liveness oracle on the SPA origin.
 *
 * Note: Traefik still routes Host(localhost) Path(`/health`) to the API.
 * This handler serves the web container's own :3000/health for in-container checks.
 */
import type { Connect, Plugin } from "vite";

export const WEB_HEALTH_PATH = "/health";
/** Node lowercases incoming header names. */
export const WEB_HEALTH_PROBE_HEADER = "octanest-health-probe";
export const WEB_HEALTH_PROBE_VALUE = "1";

export function isAuthorizedWebHealthProbe(headers: Connect.IncomingMessage["headers"]): boolean {
  const raw = headers[WEB_HEALTH_PROBE_HEADER];
  const value = Array.isArray(raw) ? raw[0] : raw;
  return value === WEB_HEALTH_PROBE_VALUE;
}

export function webHealthMiddleware(): Connect.NextHandleFunction {
  return (req, res, next) => {
    const path = (req.url ?? "").split("?")[0] ?? "";
    if (path !== WEB_HEALTH_PATH) {
      next();
      return;
    }

    if (!isAuthorizedWebHealthProbe(req.headers)) {
      res.statusCode = 404;
      res.setHeader("content-type", "text/plain; charset=utf-8");
      res.end("Not Found");
      return;
    }

    res.statusCode = 200;
    res.setHeader("cache-control", "no-store");
    res.setHeader("content-type", "application/json; charset=utf-8");
    res.end(JSON.stringify({ ok: true }));
  };
}

/** Vite plugin — runs in `vite dev` and `vite preview` (Compose web image). */
export function webHealthPlugin(): Plugin {
  const handle = webHealthMiddleware();
  return {
    name: "octanest-web-health",
    configureServer(server) {
      // Register before internal middleware so we win over `/health` → API proxy.
      server.middlewares.use(handle);
    },
    configurePreviewServer(server) {
      server.middlewares.use(handle);
    },
  };
}
