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
      ) => { fill: (v: string) => Promise<unknown> };
      waitForURL: (url: string | RegExp, opts?: object) => Promise<unknown>;
      content: () => Promise<string>;
      close: () => Promise<unknown>;
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
      waitUntil: "domcontentloaded",
    });
    await page
      .getByRole("heading", { name: "Create your account" })
      .waitFor({ state: "visible", timeout: 30_000 });
    await page.getByLabel("Email").fill(creds.email);
    await page.getByLabel("Username").fill(creds.username);
    await page.getByLabel("Password", { exact: true }).fill(creds.password);
    await page.getByLabel(/confirm password/i).fill(creds.password);
    await page.getByRole("button", { name: /create account|sign up/i }).click();
    await page.waitForURL(/\/dashboard/, { timeout: 30_000 });
    return true;
  } finally {
    await page.close();
  }
};

/** Assert WorkOS CTA is visible on /login. */
export const expectWorkosCta: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  // Drop session from prior signup so /login is not redirected to dashboard.
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
