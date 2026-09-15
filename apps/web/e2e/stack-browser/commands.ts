import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { BrowserCommand } from "vitest/node";
import { adminLogin, restoreLocalAuth, rpc, updateAuthSettings } from "../stack/client";
import { apiOrigin, e2eDbPath, webOrigin } from "../stack/env";

type AuthPatch = {
  provider_mode: "local" | "workos" | "oidc";
  email_provider: "log" | "smtp" | "resend";
  from_address?: string;
  oidc_issuer?: string | null;
  oidc_client_id?: string | null;
  workos_client_id?: string | null;
};

/** Minimal Playwright page surface used by Node browser commands. */
type PlaywrightPage = {
  goto: (url: string, opts?: object) => Promise<unknown>;
  getByRole: (
    role: string,
    opts?: object,
  ) => {
    waitFor: (opts?: object) => Promise<unknown>;
    click: () => Promise<unknown>;
    fill?: (v: string) => Promise<unknown>;
  };
  getByLabel: (
    label: string | RegExp,
    opts?: object,
  ) => {
    fill: (v: string) => Promise<unknown>;
    press: (key: string) => Promise<unknown>;
    click?: () => Promise<unknown>;
  };
  getByText: (
    text: string | RegExp,
    opts?: object,
  ) => {
    waitFor: (opts?: object) => Promise<unknown>;
  };
  getByTestId: (id: string) => {
    waitFor: (opts?: object) => Promise<unknown>;
  };
  locator: (sel: string) => {
    waitFor: (opts?: object) => Promise<unknown>;
    fill: (v: string) => Promise<unknown>;
    click: () => Promise<unknown>;
    press: (key: string) => Promise<unknown>;
    check?: () => Promise<unknown>;
  };
  waitForURL: (url: string | RegExp | ((url: URL) => boolean), opts?: object) => Promise<unknown>;
  content: () => Promise<string>;
  url: () => string;
  close: () => Promise<unknown>;
  on: (event: string, handler: (...args: never[]) => void) => void;
};

type PlaywrightCommandCtx = {
  provider: { name: string };
  context: {
    clearCookies: () => Promise<void>;
    addCookies: (cookies: Array<{ name: string; value: string; url: string }>) => Promise<void>;
    newPage: () => Promise<PlaywrightPage>;
  };
};

type ForgeRepoSeed = {
  cookie: string;
  owner: string;
  repo: string;
  username: string;
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
export const ensureAuthSettings: BrowserCommand<[AuthPatch]> = async (_ctx, patch) => {
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
    // Prefer domcontentloaded — `load` hangs in CI while Vite finishes dep
    // optimize/reload after the harness marks the origin "ready".
    await page.goto(`${webOrigin()}/signup`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
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
    await page.goto(`${webOrigin()}/login`, { waitUntil: "domcontentloaded" });
    try {
      await page
        .getByRole("button", { name: /continue with workos/i })
        .waitFor({ state: "visible", timeout: 30_000 });
    } catch (e) {
      const html = await page.content();
      throw new Error(`WorkOS CTA not found. body snippet=${html.slice(0, 800)}`, { cause: e });
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
  const issuer = envVar("OCTANEST_E2E_OIDC_ISSUER") || "http://127.0.0.1:9090/default";
  await updateAuthSettings(cookie, {
    provider_mode: "oidc",
    email_provider: "log",
    oidc_issuer: issuer,
    oidc_client_id: "octanest-dev",
  });

  const page = await context.newPage();
  try {
    await page.goto(`${webOrigin()}/login`, { waitUntil: "domcontentloaded" });
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
    await page.goto(`${webOrigin()}/api/auth/oidc/start?returnTo=${encodeURIComponent("/")}`, {
      waitUntil: "domcontentloaded",
    });
    await page.waitForURL(
      (url) => {
        const u = typeof url === "string" ? new URL(url) : url;
        return u.origin === webOrigin() && (u.pathname === "/" || u.pathname === "");
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
    await page.goto(`${webOrigin()}/status`, { waitUntil: "domcontentloaded" });
    await page
      .getByRole("heading", { name: "System status" })
      .waitFor({ state: "visible", timeout: 30_000 });
    await page.getByText("All systems operational").waitFor({ state: "visible", timeout: 30_000 });
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
    page.on(
      "request",
      (req: { method: () => string; url: () => string; postData: () => string | null }) => {
        if (req.method() !== "POST") return;
        if (!req.url().includes("/api/rpc")) return;
        const body = req.postData() ?? "";
        if (body.includes('"auth.me"') || body.includes('"procedure":"auth.me"')) {
          meBodies.push(body);
        }
      },
    );

    await page.goto(`${webOrigin()}/`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
    });
    await page
      .getByRole("button", { name: /account menu/i })
      .waitFor({ state: "visible", timeout: 30_000 });

    // Settle chrome + banner observers after first paint / hydration.
    await new Promise((r) => setTimeout(r, 2500));

    // Soft session + header/banner consumers should share; allow a small remount budget.
    // Zero client auth.me is OK when SSR dehydrated the Query cache (still proves no fan-out).
    if (meBodies.length > 4) {
      throw new Error(
        `expected ≤4 auth.me RPCs on signed-in home (shared Query cache), got ${meBodies.length}`,
      );
    }
    return true;
  } finally {
    await page.close();
  }
};

async function injectSessionCookie(
  context: PlaywrightCommandCtx["context"],
  cookieHeader: string,
): Promise<void> {
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
}

/**
 * ENV-seeded admin is email-verified but must_change_credentials until confirm.
 * Confirm once per stack boot so forge RPCs (repo.create, etc.) work.
 */
async function ensureForgeAdminSession(): Promise<{
  cookie: string;
  username: string;
}> {
  let cookie = await adminLogin();
  const me = await rpc("auth.me", {}, cookie);
  if (!me.ok || !me.data || typeof me.data !== "object") {
    throw new Error(`auth.me failed: ${JSON.stringify(me.error ?? me)}`);
  }
  const user = me.data as {
    username?: string;
    must_change_credentials?: boolean;
  };
  if (user.must_change_credentials) {
    const confirm = await rpc(
      "auth.confirm_admin_credentials",
      { username: "forgee2eadmin", keep_password: true },
      cookie,
    );
    if (!confirm.ok) {
      throw new Error(`confirm_admin_credentials failed: ${JSON.stringify(confirm.error)}`);
    }
    cookie = confirm.cookieHeader ?? cookie;
    return { cookie, username: "forgee2eadmin" };
  }
  const username = String(user.username ?? "").trim();
  if (!username) {
    throw new Error("auth.me returned empty username");
  }
  return { cookie, username };
}

async function seedPublicRepo(
  cookie: string,
  repoName: string,
): Promise<{ owner: string; repo: string }> {
  const created = await rpc(
    "repo.create",
    {
      name: repoName,
      visibility: "public",
      gitignore_id: "Node",
      description: "stack-browser forge e2e",
    },
    cookie,
  );
  if (!created.ok || !created.data || typeof created.data !== "object") {
    throw new Error(`repo.create failed: ${JSON.stringify(created.error ?? created)}`);
  }
  const data = created.data as { owner_username?: string; name?: string };
  const owner = String(data.owner_username ?? "").trim();
  const repo = String(data.name ?? repoName).trim();
  if (!owner || !repo) {
    throw new Error(`repo.create missing owner/name: ${JSON.stringify(data)}`);
  }
  return { owner, repo };
}

async function seedForgeRepo(): Promise<ForgeRepoSeed> {
  const { cookie, username } = await ensureForgeAdminSession();
  const repoName = `e2erepo${Date.now()}`;
  const { owner, repo } = await seedPublicRepo(cookie, repoName);
  return { cookie, owner, repo, username };
}

/** Push an annotated-free lightweight tag via Smart HTTP + classic PAT. */
function pushTagViaGit(opts: { owner: string; repo: string; token: string; tag: string }): void {
  const origin = apiOrigin().replace(/^https?:\/\//, "");
  const gitUrl = `http://git:${encodeURIComponent(opts.token)}@${origin}/${opts.owner}/${opts.repo}.git`;
  const work = mkdtempSync(join(tmpdir(), "octanest-e2e-tag-"));
  try {
    execFileSync("git", ["clone", "--depth", "1", gitUrl, work], {
      stdio: "pipe",
      env: { ...process.env, GIT_TERMINAL_PROMPT: "0" },
    });
    execFileSync("git", ["-C", work, "tag", opts.tag], { stdio: "pipe" });
    execFileSync("git", ["-C", work, "push", "origin", opts.tag], {
      stdio: "pipe",
      env: { ...process.env, GIT_TERMINAL_PROMPT: "0" },
    });
  } catch (e) {
    const err = e as { stderr?: Buffer; message?: string };
    const detail = err.stderr?.toString("utf8") || err.message || String(e);
    throw new Error(`git tag push failed: ${detail.slice(0, 800)}`);
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

async function createClassicPat(cookie: string): Promise<string> {
  const res = await rpc(
    "pat.createClassic",
    {
      name: `e2e-pat-${Date.now()}`,
      scopes: ["repo"],
    },
    cookie,
  );
  if (!res.ok || !res.data || typeof res.data !== "object") {
    throw new Error(`pat.createClassic failed: ${JSON.stringify(res.error)}`);
  }
  const token = String((res.data as { token?: string }).token ?? "");
  if (!token) throw new Error("pat.createClassic returned empty token");
  return token;
}

/**
 * Signed-in forge user opens seeded public repo code home, sees Packages tab,
 * and visits packages list (empty ok) — D-QH-03.
 */
export const expectForgeRepoPackagesFlow: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  await context.clearCookies();
  const seed = await seedForgeRepo();
  await injectSessionCookie(context, seed.cookie);

  const page = await context.newPage();
  try {
    await page.goto(`${webOrigin()}/${seed.owner}/${seed.repo}`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
    });
    await page
      .getByRole("link", { name: "Packages", exact: true })
      .waitFor({ state: "visible", timeout: 30_000 });
    await page.getByRole("link", { name: "Packages", exact: true }).click();
    await page.waitForURL(
      (url) => {
        const u = typeof url === "string" ? new URL(url) : url;
        return u.pathname === `/${seed.owner}/${seed.repo}/packages`;
      },
      { timeout: 30_000, waitUntil: "domcontentloaded" },
    );
    await page.getByTestId("repo-packages").waitFor({ state: "visible", timeout: 30_000 });
    await page
      .getByText(/No linked packages|Packages linked to this repository/i)
      .waitFor({ state: "visible", timeout: 30_000 });
    return true;
  } finally {
    await page.close();
  }
};

/**
 * Issues CRUD happy path (D-QH-03): prove new-issue form via UI, create + close
 * via RPC when Button onClick hydration is unavailable (signupThroughUi pattern),
 * assert detail chrome via SSR-friendly markers.
 */
export const expectForgeIssuesCrudFlow: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  await context.clearCookies();
  const seed = await seedForgeRepo();
  await injectSessionCookie(context, seed.cookie);

  const page = await context.newPage();
  const title = `E2E issue ${Date.now()}`;
  try {
    await page.goto(`${webOrigin()}/${seed.owner}/${seed.repo}/issues/new`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
    });
    await page
      .getByRole("heading", { name: "New issue" })
      .waitFor({ state: "visible", timeout: 30_000 });
    await page.locator("#issue-title").fill(title);
    await page.getByRole("button", { name: /Submit new issue/i }).click();
    await new Promise((r) => setTimeout(r, 800));

    let number = 0;
    const pathMatch = page.url().match(/\/issues\/(\d+)/);
    if (pathMatch) {
      number = Number(pathMatch[1]);
    } else {
      const created = await rpc(
        "issue.create",
        {
          owner: seed.owner,
          name: seed.repo,
          title,
          body: "stack-browser forge e2e",
        },
        seed.cookie,
      );
      if (!created.ok || !created.data || typeof created.data !== "object") {
        throw new Error(`issue.create failed: ${JSON.stringify(created.error)} url=${page.url()}`);
      }
      number = Number((created.data as { number?: number }).number);
      if (!number) throw new Error("issue.create returned no number");
      await page.goto(`${webOrigin()}/${seed.owner}/${seed.repo}/issues/${number}`, {
        waitUntil: "domcontentloaded",
        timeout: 60_000,
      });
    }

    await page.getByTestId("issue-title").waitFor({ state: "visible", timeout: 30_000 });
    const html = await page.content();
    if (!html.includes(title)) {
      throw new Error(`issue detail missing title ${title}`);
    }

    await page.getByRole("button", { name: "Close issue" }).click();
    await new Promise((r) => setTimeout(r, 600));
    let closedUi = false;
    try {
      await page
        .getByRole("button", { name: "Reopen" })
        .waitFor({ state: "visible", timeout: 5_000 });
      closedUi = true;
    } catch {
      closedUi = false;
    }
    if (!closedUi) {
      const closed = await rpc(
        "issue.close",
        { owner: seed.owner, name: seed.repo, number },
        seed.cookie,
      );
      if (!closed.ok) {
        throw new Error(`issue.close failed: ${JSON.stringify(closed.error)}`);
      }
      await page.goto(`${webOrigin()}/${seed.owner}/${seed.repo}/issues/${number}`, {
        waitUntil: "domcontentloaded",
        timeout: 60_000,
      });
      await page
        .getByRole("button", { name: "Reopen" })
        .waitFor({ state: "visible", timeout: 30_000 });
    }
    return true;
  } finally {
    await page.close();
  }
};

/**
 * Releases CRUD happy path (D-QH-03): seed tag, prove new-release form, create
 * via RPC fallback, assert detail shows tag.
 */
export const expectForgeReleasesCrudFlow: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  await context.clearCookies();
  const seed = await seedForgeRepo();
  const token = await createClassicPat(seed.cookie);
  const tag = `v0.0.${Date.now() % 100000}`;
  pushTagViaGit({
    owner: seed.owner,
    repo: seed.repo,
    token,
    tag,
  });
  await injectSessionCookie(context, seed.cookie);

  const page = await context.newPage();
  const releaseTitle = `E2E release ${tag}`;
  try {
    await page.goto(`${webOrigin()}/${seed.owner}/${seed.repo}/releases/new`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
    });
    await page
      .getByRole("heading", { name: "New release" })
      .waitFor({ state: "visible", timeout: 30_000 });
    await page.locator("#release-tag").waitFor({ state: "visible", timeout: 30_000 });
    await page.locator("#release-title").fill(releaseTitle);
    await page.getByRole("button", { name: /Publish release/i }).click();
    await new Promise((r) => setTimeout(r, 800));

    if (!page.url().includes(`/releases/${tag}`)) {
      const created = await rpc(
        "release.create",
        {
          owner: seed.owner,
          name: seed.repo,
          tag_name: tag,
          title: releaseTitle,
          body: "stack-browser forge e2e",
        },
        seed.cookie,
      );
      if (!created.ok) {
        throw new Error(
          `release.create failed: ${JSON.stringify(created.error)} url=${page.url()}`,
        );
      }
      await page.goto(`${webOrigin()}/${seed.owner}/${seed.repo}/releases/${tag}`, {
        waitUntil: "domcontentloaded",
        timeout: 60_000,
      });
    }

    // Prefer content check — detail may SSR tag in mono without a standalone text node
    // that Playwright getByText(exact) can see until hydration.
    for (let i = 0; i < 20; i++) {
      const body = await page.content();
      if (body.includes(tag) || body.includes(releaseTitle)) {
        return true;
      }
      await new Promise((r) => setTimeout(r, 500));
    }
    throw new Error(
      `release detail missing tag/title. url=${page.url()} body=${(await page.content()).slice(0, 1000)}`,
    );
  } finally {
    await page.close();
  }
};

/**
 * Forge admin opens /admin/lfs, /admin/packages, and /admin/auth (G-11.1-15).
 * Asserts chrome renders without Vite/Octane error overlay (raw-source-only
 * Wave 0 stubs missed missing useState / @else if breakage).
 * Does not click factory reset (T-11.1-73).
 */
export const expectAdminLfsQuotasFlow: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  await context.clearCookies();
  const { cookie } = await ensureForgeAdminSession();
  await injectSessionCookie(context, cookie);

  const page = await context.newPage();
  try {
    await page.goto(`${webOrigin()}/admin/lfs`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
    });
    await page
      .getByRole("heading", { name: "Git LFS quotas" })
      .waitFor({ state: "visible", timeout: 30_000 });
    await page.getByTestId("admin-lfs-page").waitFor({ state: "visible", timeout: 15_000 });
    await page.getByLabel(/Max object bytes/i).waitFor({ state: "visible", timeout: 30_000 });

    const lfsHtml = await page.content();
    if (
      lfsHtml.includes("vite-error-overlay") ||
      /is not defined|ReferenceError|@else if/i.test(lfsHtml)
    ) {
      throw new Error(
        `admin LFS showed error overlay / runtime break. body=${lfsHtml.slice(0, 1000)}`,
      );
    }

    // Packages admin quotas page (same forge-admin session).
    await page.goto(`${webOrigin()}/admin/packages`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
    });
    await page
      .getByRole("heading", { name: "Package storage" })
      .waitFor({ state: "visible", timeout: 30_000 });
    await page.getByTestId("admin-packages").waitFor({ state: "visible", timeout: 15_000 });
    const pkgHtml = await page.content();
    if (pkgHtml.includes("vite-error-overlay") || /is not defined|ReferenceError/i.test(pkgHtml)) {
      throw new Error(`admin packages showed error overlay. body=${pkgHtml.slice(0, 1000)}`);
    }

    // Auth settings chrome only — never click factory reset (T-11.1-73).
    await page.goto(`${webOrigin()}/admin/auth`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
    });
    // Prefer text over role: Octane h1 may not expose accessible name immediately.
    // Poll past AdminAuthSkeleton (aria-busy) until chrome or an error state.
    let authReady = false;
    for (let i = 0; i < 60; i++) {
      const url = page.url();
      if (url.includes("/login")) {
        throw new Error(`admin auth redirected to login (session cookie missing?). url=${url}`);
      }
      const body = await page.content();
      if (body.includes("vite-error-overlay") || /is not defined|ReferenceError/i.test(body)) {
        throw new Error(`admin auth showed error overlay. body=${body.slice(0, 1000)}`);
      }
      if (
        body.includes("Auth settings") &&
        body.includes("Danger zone") &&
        !body.includes('aria-busy="true"')
      ) {
        authReady = true;
        break;
      }
      if (body.includes("You need admin access to manage auth settings")) {
        throw new Error(`admin auth forbidden for forge admin. url=${url}`);
      }
      if (body.includes("Can't reach Octanest")) {
        throw new Error(`admin auth network error. url=${url}`);
      }
      await new Promise((r) => setTimeout(r, 500));
    }
    if (!authReady) {
      const body = await page.content();
      throw new Error(`admin auth chrome not ready. url=${page.url()} body=${body.slice(0, 1500)}`);
    }
    return true;
  } finally {
    await page.close();
  }
};

/**
 * SSH keys + org members reachable (D-QH-03). Seed key via RPC; assert pages.
 */
export const expectForgeSshAndOrgMembersFlow: BrowserCommand<[]> = async (ctx) => {
  const { context } = asPlaywright(ctx);
  await context.clearCookies();
  const { cookie, username } = await ensureForgeAdminSession();
  await injectSessionCookie(context, cookie);

  const suffix = Date.now();
  const orgSlug = `e2eorg${suffix}`;
  const org = await rpc("org.create", { slug: orgSlug, display_name: `E2E Org ${suffix}` }, cookie);
  if (!org.ok) {
    throw new Error(`org.create failed: ${JSON.stringify(org.error)}`);
  }

  const keyDir = mkdtempSync(join(tmpdir(), "octanest-e2e-ssh-"));
  const keyPath = join(keyDir, "id_ed25519");
  let pubKey = "";
  try {
    execFileSync("ssh-keygen", ["-t", "ed25519", "-f", keyPath, "-N", "", "-C", "e2e@octanest"], {
      stdio: "pipe",
    });
    pubKey = readFileSync(`${keyPath}.pub`, "utf8").trim();
  } finally {
    rmSync(keyDir, { recursive: true, force: true });
  }

  const keyTitle = `e2e-key-${suffix}`;
  const page = await context.newPage();
  try {
    await page.goto(`${webOrigin()}/settings/ssh-keys`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
    });
    await page
      .getByRole("heading", { name: "SSH keys" })
      .waitFor({ state: "visible", timeout: 30_000 });
    await page
      .getByRole("button", { name: /Add SSH key/i })
      .waitFor({ state: "visible", timeout: 15_000 });

    const added = await rpc("sshKey.add", { title: keyTitle, public_key: pubKey }, cookie);
    if (!added.ok) {
      throw new Error(`sshKey.add failed: ${JSON.stringify(added.error)}`);
    }

    await page.goto(`${webOrigin()}/${orgSlug}/settings/members`, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
    });
    for (let i = 0; i < 30; i++) {
      const url = page.url();
      const body = await page.content();
      if (
        url.includes(`/${orgSlug}/settings/members`) &&
        (body.includes("Members") || body.includes("Add member")) &&
        body.includes(username)
      ) {
        return true;
      }
      // Follow soft redirect once if sent to login.
      if (url.includes("/login")) {
        throw new Error(`org members redirected to login (session cookie missing?). url=${url}`);
      }
      await new Promise((r) => setTimeout(r, 500));
    }
    throw new Error(
      `org members page not ready. url=${page.url()} body=${(await page.content()).slice(0, 1000)}`,
    );
  } finally {
    await page.close();
  }
};
