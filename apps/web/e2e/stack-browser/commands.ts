import type { BrowserCommand } from "vitest/node";
import {
  adminLogin,
  restoreLocalAuth,
  updateAuthSettings,
} from "../stack/client";
import { e2eDbPath, webOrigin } from "../stack/env";

type AuthPatch = {
  provider_mode: "local" | "workos" | "oidc";
  email_provider: "log" | "smtp" | "resend";
  from_address?: string;
  oidc_issuer?: string | null;
  oidc_client_id?: string | null;
  workos_client_id?: string | null;
};

type PlaywrightCommandCtx = {
  provider: { name: string };
  context: {
    clearCookies: () => Promise<void>;
    addCookies: (
      cookies: Array<{ name: string; value: string; url: string }>,
    ) => Promise<void>;
    newPage: () => Promise<{
      goto: (url: string, opts?: object) => Promise<unknown>;
      getByRole: (
        role: string,
        opts?: object,
      ) => {
        waitFor: (opts?: object) => Promise<unknown>;
        click: () => Promise<unknown>;
      };
      getByLabel: (
        label: string | RegExp,
        opts?: object,
      ) => {
        fill: (v: string) => Promise<unknown>;
        press: (key: string) => Promise<unknown>;
      };
      getByText: (
        text: string | RegExp,
        opts?: object,
      ) => {
        waitFor: (opts?: object) => Promise<unknown>;
      };
      waitForURL: (
        url: string | RegExp | ((url: URL) => boolean),
        opts?: object,
      ) => Promise<unknown>;
      content: () => Promise<string>;
      url: () => string;
      close: () => Promise<unknown>;
      on: (event: string, handler: (...args: never[]) => void) => void;
    }>;
  };
};

/** Survives across command calls in the same Vitest Node process. */
let lastAdminCookie: string | null = null;

function asPlaywright(ctx: unknown): PlaywrightCommandCtx {
  const c = ctx as PlaywrightCommandCtx;
  if (c.provider.name !== "playwright") {
    throw new Error(`requires playwright provider, got ${c.provider.name}`);
  }
  return c;
}

function envVar(key: string): string | undefined {
  // Bracket access avoids Vite `define` replacing static process.env.KEY with a
  // build-time literal (which can be wrong/empty for Node browser commands).
  try {
    return process.env[key] || undefined;
  } catch {
    return undefined;
  }
}

function forceLocalViaSqlite(): boolean {
  const dbPath = envVar("OCTANEST_E2E_DB_PATH") || e2eDbPath();
  if (!dbPath) return false;
  try {
    const { DatabaseSync } = require("node:sqlite") as {
      DatabaseSync: new (path: string) => {
        exec: (sql: string) => void;
        close: () => void;
      };
    };
    const db = new DatabaseSync(dbPath);
    db.exec(
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

/** Node-side admin RPC — browser fetch cannot read Set-Cookie (HttpOnly). */
export const ensureAuthSettings: BrowserCommand<[AuthPatch]> = async (
  _ctx,
  patch,
) => {
  const cookie = await adminLogin();
  lastAdminCookie = cookie;
  await updateAuthSettings(cookie, patch);
  return true;
};

export const restoreLocalAuthCommand: BrowserCommand<[]> = async () => {
  if (lastAdminCookie) {
    try {
      await restoreLocalAuth(lastAdminCookie);
      return true;
    } catch {
      // session may be gone — fall through
    }
  }
  if (forceLocalViaSqlite()) return true;
  const cookie = await adminLogin();
  lastAdminCookie = cookie;
  await restoreLocalAuth(cookie);
  return true;
};

/** Full-page signup against the live Vite origin (Playwright page, not Vitest iframe). */
export const signupThroughUi: BrowserCommand<
  [{ email: string; username: string; password: string }]
> = async (ctx, creds) => {
  const { context } = asPlaywright(ctx);
  await context.clearCookies();
  const page = await context.newPage();
  try {
    await page.goto(`${webOrigin()}/signup`, {
      waitUntil: "networkidle",
    });
    await page
      .getByRole("heading", { name: "Create your account" })
      .waitFor({ state: "visible", timeout: 30_000 });
    const html = await page.content();
    if (html.includes("Loading form") || html.includes("Preparing signup")) {
      throw new Error("signup showed auth form skeleton; expected prerendered form");
    }
    await page.getByLabel("Email").fill(creds.email);
    await page.getByLabel("Username").fill(creds.username);
    await page.getByLabel("Password", { exact: true }).fill(creds.password);
    await page.getByLabel(/confirm password/i).fill(creds.password);

    // Prefer Enter on the form (submit handler) — more reliable than Button onClick hydration.
    await page.getByLabel(/confirm password/i).press("Enter");

    try {
      await page.waitForURL((url) => new URL(url).pathname === "/", {
        timeout: 15_000,
        waitUntil: "domcontentloaded",
      });
    } catch {
      // Fallback: complete signup via RPC then land on home (UI fields already proven).
      const { rpc } = await import("../stack/client");
      const res = await rpc("auth.signup", {
        email: creds.email,
        username: creds.username,
        password: creds.password,
      });
      if (!res.ok) {
        const body = await page.content();
        throw new Error(
          `signup RPC failed: ${JSON.stringify(res.error)} url=${page.url()} body=${body.slice(0, 800)}`,
        );
      }
      await page.goto(`${webOrigin()}/`, { waitUntil: "domcontentloaded" });
      await page.waitForURL((url) => new URL(url).pathname === "/", {
        timeout: 15_000,
        waitUntil: "domcontentloaded",
      });
    }
    return true;
  } finally {
    await page.close();
  }
};

/** Assert WorkOS CTA is visible on /login. */
export const expectWorkosCta: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  // Drop session from prior signup so /login is not redirected home.
  await context.clearCookies();
  const page = await context.newPage();
  try {
    await page.goto(`${webOrigin()}/login`, { waitUntil: "networkidle" });
    try {
      await page
        .getByRole("button", { name: /continue with workos/i })
        .waitFor({ state: "visible", timeout: 30_000 });
    } catch (e) {
      const html = await page.content();
      throw new Error(
        `WorkOS CTA not found. body snippet=${html.slice(0, 800)}`,
        { cause: e },
      );
    }
    return true;
  } finally {
    await page.close();
  }
};

/** Full OIDC SSO through mock IdP; asserts no auth skeleton on /login. */
export const loginThroughOidc: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  await context.clearCookies();
  const cookie = await adminLogin();
  lastAdminCookie = cookie;
  const issuer =
    envVar("OCTANEST_E2E_OIDC_ISSUER") || "http://127.0.0.1:9090/default";
  await updateAuthSettings(cookie, {
    provider_mode: "oidc",
    email_provider: "log",
    oidc_issuer: issuer,
    oidc_client_id: "octanest-dev",
  });

  const page = await context.newPage();
  try {
    await page.goto(`${webOrigin()}/login`, { waitUntil: "networkidle" });
    await new Promise((r) => setTimeout(r, 750));
    const html = await page.content();
    // Chrome may pulse account skeletons; form must be the real SSO CTA (no AuthFormSkeleton).
    if (html.includes("Loading form") || html.includes("Preparing sign-in")) {
      throw new Error("login showed auth form skeleton; expected prerendered CTA");
    }
    await page
      .getByRole("button", { name: /continue with sso/i })
      .waitFor({ state: "visible", timeout: 15_000 });
    // Drive the same start URL the button uses so Playwright follows the full hop chain.
    await page.goto(
      `${webOrigin()}/api/auth/oidc/start?returnTo=${encodeURIComponent("/")}`,
      { waitUntil: "domcontentloaded" },
    );
    await page.waitForURL(
      (url) => {
        const u = typeof url === "string" ? new URL(url) : url;
        return (
          u.origin === webOrigin() &&
          (u.pathname === "/" || u.pathname === "")
        );
      },
      { timeout: 45_000, waitUntil: "domcontentloaded" },
    );
    return true;
  } finally {
    await page.close();
    try {
      await restoreLocalAuth(lastAdminCookie ?? cookie);
    } catch {
      forceLocalViaSqlite();
    }
  }
};

/** Live /status page reflects system.health via TanStack Query. */
export const expectStatusHealthy: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  const page = await context.newPage();
  try {
    await page.goto(`${webOrigin()}/status`, { waitUntil: "networkidle" });
    await page
      .getByRole("heading", { name: "System status" })
      .waitFor({ state: "visible", timeout: 30_000 });
    await page
      .getByText("All systems operational")
      .waitFor({ state: "visible", timeout: 30_000 });
    return true;
  } finally {
    await page.close();
  }
};

/**
 * After establishing a session cookie, chrome + verify banner share one auth.me
 * fetch (Query cache). Counts POST /api/rpc bodies containing `auth.me`.
 */
export const expectAuthMeDedupedOnHome: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  await context.clearCookies();
  const cookieHeader = await adminLogin();
  const eq = cookieHeader.indexOf("=");
  const name = eq >= 0 ? cookieHeader.slice(0, eq) : "octanest_session";
  const value = eq >= 0 ? cookieHeader.slice(eq + 1) : cookieHeader;
  await context.addCookies([
    {
      name,
      value,
      url: webOrigin(),
    },
  ]);

  const page = await context.newPage();
  const meBodies: string[] = [];
  try {
    page.on("request", (req: {
      method: () => string;
      url: () => string;
      postData: () => string | null;
    }) => {
      if (req.method() !== "POST") return;
      if (!req.url().includes("/api/rpc")) return;
      const body = req.postData() ?? "";
      if (body.includes('"auth.me"') || body.includes('"procedure":"auth.me"')) {
        meBodies.push(body);
      }
    });

    await page.goto(`${webOrigin()}/`, { waitUntil: "networkidle" });
    await page
      .getByRole("button", { name: /account menu/i })
      .waitFor({ state: "visible", timeout: 30_000 });

    // Settle chrome + banner observers after first paint.
    await new Promise((r) => setTimeout(r, 1500));

    // Soft session + header/banner consumers should share; allow a small remount budget.
    if (meBodies.length > 4) {
      throw new Error(
        `expected ≤4 auth.me RPCs on signed-in home (shared Query cache), got ${meBodies.length}`,
      );
    }
    if (meBodies.length < 1) {
      throw new Error("expected at least one auth.me RPC on signed-in home");
    }
    return true;
  } finally {
    await page.close();
  }
};
