#!/usr/bin/env bun
/**
 * Local throwaway stubs for Resend + WorkOS AuthKit (no cloud secrets).
 * Exposes /__test/* for stack e2e assertions. See docs/dev-auth.md
 */
const port = Number(process.env.PORT || 9092);

/** @type {{ at: string, method: string, path: string, body?: unknown }[]} */
const log = [];

function json(data, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "content-type": "application/json" },
  });
}

function redirect(location) {
  return new Response(null, { status: 302, headers: { location } });
}

function record(method, path, body) {
  log.push({ at: new Date().toISOString(), method, path, body });
  if (log.length > 200) log.shift();
}

const server = Bun.serve({
  port,
  hostname: "0.0.0.0",
  async fetch(req) {
    const url = new URL(req.url);

    if (url.pathname === "/__test/reset" && req.method === "POST") {
      log.length = 0;
      return json({ ok: true });
    }
    if (url.pathname === "/__test/log" && req.method === "GET") {
      return json({ ok: true, entries: log });
    }

    // --- Resend ---
    if (req.method === "POST" && url.pathname === "/emails") {
      let body = null;
      try {
        body = await req.json();
      } catch {
        body = null;
      }
      record("POST", "/emails", body);
      console.log(
        `[resend] POST /emails to=${JSON.stringify(body?.to)} subject=${body?.subject}`,
      );
      return json({ id: "email_dev_local" });
    }

    // --- WorkOS AuthKit authorize (browser) ---
    if (req.method === "GET" && url.pathname === "/user_management/authorize") {
      const redirectUri = url.searchParams.get("redirect_uri");
      const state = url.searchParams.get("state") || "";
      record("GET", "/user_management/authorize", {
        redirect_uri: redirectUri,
        state,
        client_id: url.searchParams.get("client_id"),
      });
      if (!redirectUri) {
        return json({ error: "missing redirect_uri" }, 400);
      }
      const target = new URL(redirectUri);
      target.searchParams.set("code", "dev_auth_code");
      target.searchParams.set("state", state);
      console.log(`[workos] authorize → ${target}`);
      return redirect(target.toString());
    }

    // --- WorkOS authenticate (API) ---
    if (req.method === "POST" && url.pathname === "/user_management/authenticate") {
      let body = null;
      try {
        body = await req.json();
      } catch {
        body = null;
      }
      record("POST", "/user_management/authenticate", body);
      console.log("[workos] authenticate_with_code");
      const now = new Date().toISOString();
      return json({
        user: {
          object: "user",
          id: "user_dev_local",
          email: "dev@oxidean.local",
          email_verified: true,
          first_name: "Dev",
          last_name: "User",
          name: "Dev User",
          created_at: now,
          updated_at: now,
        },
        access_token: "access_dev_local",
        refresh_token: "refresh_dev_local",
      });
    }

    if (url.pathname === "/health" || url.pathname === "/") {
      return json({ ok: true, service: "oxidean-dev-stubs" });
    }

    return json({ error: "not_found", path: url.pathname }, 404);
  },
});

console.log(`oxidean-dev-stubs listening on http://0.0.0.0:${server.port}`);
