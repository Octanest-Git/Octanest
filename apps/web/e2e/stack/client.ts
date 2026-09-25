import { adminEmail, adminPassword, apiOrigin, e2eDbPath } from "./env.ts";

export type RpcResult = {
  status: number;
  ok: boolean;
  data?: unknown;
  error?: { code?: string; message?: string };
  setCookie?: string;
  cookieHeader?: string;
};

function cookiePair(setCookie: string | undefined): string | undefined {
  if (!setCookie) return undefined;
  return setCookie.split(";")[0]?.trim();
}

export async function rpc(
  procedure: string,
  input: Record<string, unknown> = {},
  cookie?: string,
): Promise<RpcResult> {
  const headers: Record<string, string> = {
    "content-type": "application/json",
    "Oxidean-RPC-Version": "1",
  };
  if (cookie) headers.cookie = cookie;

  const res = await fetch(`${apiOrigin()}/api/rpc`, {
    method: "POST",
    headers,
    body: JSON.stringify({ procedure, input }),
    redirect: "manual",
  });

  const setCookie = res.headers.get("set-cookie") ?? undefined;
  const body = (await res.json()) as {
    ok?: boolean;
    data?: unknown;
    error?: { code?: string; message?: string };
  };

  return {
    status: res.status,
    ok: body.ok === true,
    data: body.data,
    error: body.error,
    setCookie,
    cookieHeader: cookiePair(setCookie),
  };
}

/** Node-only: force local provider when a prior SSO test left mode ≠ local. */
async function forceLocalProviderViaSqlite(): Promise<boolean> {
  const dbPath = e2eDbPath();
  if (!dbPath) return false;
  try {
    // Prefer node:sqlite (Vitest commands run in Node, not Bun).
    const { DatabaseSync } = await import("node:sqlite");
    const db = new DatabaseSync(dbPath);
    db.exec(
      `UPDATE instance_auth_settings
       SET provider_mode = 'local', email_provider = 'log'
       WHERE id = 1`,
    );
    db.close();
    return true;
  } catch {
    try {
      const { Database } = await import("bun:sqlite");
      const db = new Database(dbPath);
      db.run(
        `UPDATE instance_auth_settings
         SET provider_mode = 'local', email_provider = 'log'
         WHERE id = 1`,
      );
      db.close();
      return true;
    } catch {
      return false;
    }
  }
}

export async function adminLogin(): Promise<string> {
  const attempt = async () =>
    rpc("auth.login", {
      identifier: adminEmail(),
      password: adminPassword(),
      remember_me: false,
    });

  let res = await attempt();
  if (
    !res.ok &&
    res.error?.code === "auth.provider_mismatch" &&
    (await forceLocalProviderViaSqlite())
  ) {
    res = await attempt();
  }
  if (!res.ok || !res.cookieHeader) {
    throw new Error(`admin login failed: status=${res.status} error=${JSON.stringify(res.error)}`);
  }
  return res.cookieHeader;
}

export async function updateAuthSettings(
  cookie: string,
  patch: {
    provider_mode: "local" | "workos" | "oidc";
    email_provider: "log" | "smtp" | "resend";
    from_address?: string;
    oidc_issuer?: string | null;
    oidc_client_id?: string | null;
    workos_client_id?: string | null;
    allow_signup?: boolean;
  },
): Promise<void> {
  const res = await rpc(
    "admin.auth.update_settings",
    {
      provider_mode: patch.provider_mode,
      email_provider: patch.email_provider,
      from_address: patch.from_address ?? "Oxidean <noreply@localhost>",
      oidc_issuer: patch.oidc_issuer ?? null,
      oidc_client_id: patch.oidc_client_id ?? null,
      workos_client_id: patch.workos_client_id ?? null,
      // Seeded ENV admin defaults allow_signup=false; stack signup tests need it on.
      allow_signup: patch.allow_signup ?? true,
    },
    cookie,
  );
  if (!res.ok) {
    throw new Error(`update_settings failed: ${JSON.stringify(res.error ?? res)}`);
  }
}

export async function restoreLocalAuth(cookie: string): Promise<void> {
  await updateAuthSettings(cookie, {
    provider_mode: "local",
    email_provider: "log",
  });
}

/** Login as admin, run work, always restore local mode afterward. */
export async function withAdminSession<T>(fn: (cookie: string) => Promise<T>): Promise<T> {
  const cookie = await adminLogin();
  try {
    return await fn(cookie);
  } finally {
    try {
      await restoreLocalAuth(cookie);
    } catch {
      // best-effort — next adminLogin may SQLite-reset
    }
  }
}

/** Follow redirects manually (WorkOS/OIDC), collecting Set-Cookie. */
export async function followRedirects(
  startUrl: string,
  opts: { maxHops?: number; cookie?: string } = {},
): Promise<{
  finalUrl: string;
  status: number;
  cookies: string[];
  bodyText: string;
}> {
  const maxHops = opts.maxHops ?? 12;
  let url = startUrl;
  const cookies: string[] = [];
  let cookieHeader = opts.cookie ?? "";

  for (let i = 0; i < maxHops; i++) {
    const headers: Record<string, string> = {};
    if (cookieHeader) headers.cookie = cookieHeader;

    const res = await fetch(url, {
      method: "GET",
      headers,
      redirect: "manual",
      signal: AbortSignal.timeout(15_000),
    });

    const sc = res.headers.getSetCookie?.() ?? [];
    if (sc.length === 0) {
      const single = res.headers.get("set-cookie");
      if (single) sc.push(single);
    }
    for (const raw of sc) {
      const pair = raw.split(";")[0]?.trim();
      if (pair) {
        cookies.push(pair);
        // Merge/replace by name for subsequent hops
        const name = pair.split("=")[0];
        const parts = cookieHeader
          .split(";")
          .map((p) => p.trim())
          .filter(Boolean)
          .filter((p) => !p.startsWith(`${name}=`));
        parts.push(pair);
        cookieHeader = parts.join("; ");
      }
    }

    if (res.status >= 300 && res.status < 400) {
      const loc = res.headers.get("location");
      if (!loc) {
        return {
          finalUrl: url,
          status: res.status,
          cookies,
          bodyText: await res.text(),
        };
      }
      url = new URL(loc, url).toString();
      continue;
    }

    return {
      finalUrl: url,
      status: res.status,
      cookies,
      bodyText: await res.text(),
    };
  }

  throw new Error(`too many redirects from ${startUrl}`);
}
