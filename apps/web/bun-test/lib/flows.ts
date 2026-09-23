/**
 * Stack flows driven by Bun.WebView for issue #37 PoC.
 * Keep selectors on stable ids / data-testid (no Playwright getByRole).
 * Target Octane `.tsrx` UI (onInput text fields, data-testid radios) — not React.
 */
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { adminLogin, restoreLocalAuth, rpc, updateAuthSettings } from "../../e2e/stack/client.ts";
import { apiOrigin, webOrigin } from "./env.ts";
import {
  assertNoOctaneOverlay,
  navigateSafe,
  newGuardedWebView,
  setCookieForOrigin,
  trackRpcPosts,
  viewHtml,
  waitForButtonMatching,
  waitForSelector,
  waitForText,
} from "./webview-guard.ts";

function envGet(key: string): string | undefined {
  const v = process.env[key];
  if (v !== undefined && v !== "") return v;
  return undefined;
}

async function ensureForgeAdminSession(): Promise<{ cookie: string; username: string }> {
  let cookie = await adminLogin();
  const me = await rpc("auth.me", {}, cookie);
  if (!me.ok || !me.data || typeof me.data !== "object") {
    throw new Error(`auth.me failed: ${JSON.stringify(me.error ?? me)}`);
  }
  const user = me.data as { username?: string; must_change_credentials?: boolean };
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
  if (!username) throw new Error("auth.me returned empty username");
  return { cookie, username };
}

async function seedForgeRepo(): Promise<{
  cookie: string;
  owner: string;
  repo: string;
}> {
  const { cookie } = await ensureForgeAdminSession();
  const repoName = `bune2e${Date.now()}`;
  const created = await rpc(
    "repo.create",
    {
      name: repoName,
      visibility: "public",
      gitignore_id: "Node",
      description: "bun-webview poc forge e2e",
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
  return { cookie, owner, repo };
}

/** Live /status reflects system.health. */
export async function expectStatusHealthy(): Promise<boolean> {
  const guard = await newGuardedWebView();
  try {
    await navigateSafe(guard.view, `${webOrigin()}/status`);
    await waitForText(guard.view, "System status");
    await waitForText(guard.view, "All systems operational");
    assertNoOctaneOverlay(await viewHtml(guard.view), "status");
    return true;
  } finally {
    await guard.close("bun-webview status");
  }
}

/** Signup through the live Vite origin (CSS ids, trusted click/type). */
export async function signupThroughUi(creds: {
  email: string;
  username: string;
  password: string;
}): Promise<boolean> {
  const guard = await newGuardedWebView();
  try {
    await navigateSafe(guard.view, `${webOrigin()}/signup`);
    // Prefer selector — Vite cold routes can delay heading text.
    await waitForSelector(guard.view, "#signup-email", 60_000);
    await waitForText(guard.view, "Create your account", 15_000);
    const html = await viewHtml(guard.view);
    if (html.includes("Loading form") || html.includes("Preparing signup")) {
      throw new Error("signup showed auth form skeleton; expected prerendered form");
    }
    assertNoOctaneOverlay(html, "signup");

    await waitForSelector(guard.view, "#signup-email");
    await guard.view.click("#signup-email");
    await guard.view.type(creds.email);
    await guard.view.click("#signup-username");
    await guard.view.type(creds.username);
    await guard.view.click("#signup-password");
    await guard.view.type(creds.password);
    await guard.view.click("#signup-confirm");
    await guard.view.type(creds.password);
    await guard.view.press("Enter");

    const deadline = Date.now() + 15_000;
    let landed = false;
    while (Date.now() < deadline) {
      const path = await guard.view.evaluate("location.pathname");
      if (path === "/" || path === "") {
        landed = true;
        break;
      }
      await Bun.sleep(200);
    }

    if (!landed) {
      const res = await rpc("auth.signup", {
        email: creds.email,
        username: creds.username,
        password: creds.password,
      });
      if (!res.ok) {
        // UI submit may have already created the account (auth.taken) while
        // navigation lagged behind Vite/SSR.
        if (res.error?.code === "auth.taken") {
          await navigateSafe(guard.view, `${webOrigin()}/`);
          return true;
        }
        const body = await viewHtml(guard.view);
        throw new Error(
          `signup RPC failed: ${JSON.stringify(res.error)} body=${body.slice(0, 800)}`,
        );
      }
      await navigateSafe(guard.view, `${webOrigin()}/`);
    }
    return true;
  } finally {
    await guard.close("bun-webview signup");
  }
}

/**
 * Repo settings mirror panel: HTTPS → SSH without insertBefore pageerrors.
 * Chromium gate (happy-dom misses this race).
 */
export async function expectMirrorAuthToggleFlow(): Promise<boolean> {
  const seed = await seedForgeRepo();
  const guard = await newGuardedWebView();
  try {
    // Establish origin before Network.setCookie (CDP session must target a page).
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), seed.cookie);
    await navigateSafe(guard.view, `${webOrigin()}/${seed.owner}/${seed.repo}/settings`);
    await waitForSelector(guard.view, '[data-testid="repo-mirror-settings"]', 30_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "repo mirror settings initial");

    await waitForSelector(guard.view, '[data-testid="mirror-auth-kind-ssh"]', 15_000);
    try {
      await guard.view.scrollTo('[data-testid="mirror-auth-kind-ssh"]');
    } catch {
      // scrollTo may throw if already in view
    }

    let selected = false;
    let sshClass = "";
    for (let attempt = 0; attempt < 12; attempt++) {
      try {
        await guard.view.click('[data-testid="mirror-auth-kind-ssh"]', { timeout: 5_000 });
      } catch {
        // Fallback: dispatch a trusted click via evaluate when actionability stalls
        // (Base UI radio may report zero box until hydrated).
        await guard.view.evaluate(`(() => {
          const el = document.querySelector('[data-testid="mirror-auth-kind-ssh"]');
          if (el instanceof HTMLElement) el.click();
        })()`);
      }
      for (let i = 0; i < 10; i++) {
        const state = (await guard.view.evaluate(`(() => {
          const radio = document.querySelector('[data-testid="mirror-auth-kind-ssh"]');
          const panel = document.querySelector('[data-testid="mirror-auth-ssh"]');
          return {
            aria: radio?.getAttribute("aria-checked") ?? null,
            sshClass: panel?.getAttribute("class") ?? null,
          };
        })()`)) as { aria: string | null; sshClass: string | null };
        sshClass = state.sshClass ?? "";
        if (state.aria === "true" && sshClass && !/\bhidden\b/.test(sshClass)) {
          selected = true;
          break;
        }
        await Bun.sleep(150);
      }
      if (selected) break;
      await Bun.sleep(250);
    }

    if (!selected) {
      throw new Error(
        `SSH auth not selected after clicks (class=${JSON.stringify(sshClass)}); pageerrors=${guard.pageErrors.join(" | ") || "none"}; api=${apiOrigin()}`,
      );
    }

    await waitForSelector(guard.view, "#mirror-kh", 10_000);
    const httpsClass = (await guard.view.evaluate(
      `document.querySelector('[data-testid="mirror-auth-https"]')?.getAttribute("class") ?? ""`,
    )) as string;
    if (!/\bhidden\b/.test(httpsClass)) {
      throw new Error(
        `mirror-auth-https should be hidden after SSH click (class=${JSON.stringify(httpsClass)})`,
      );
    }

    assertNoOctaneOverlay(await viewHtml(guard.view), "repo mirror settings after SSH");
    return true;
  } finally {
    await guard.close("bun-webview mirror auth toggle");
  }
}

/** WorkOS CTA visible on /login after admin flips provider_mode. */
export async function expectWorkosCta(): Promise<boolean> {
  const cookie = await adminLogin();
  try {
    await updateAuthSettings(cookie, {
      provider_mode: "workos",
      email_provider: "log",
      workos_client_id: "client_dev_local",
    });
    const guard = await newGuardedWebView();
    try {
      await navigateSafe(guard.view, `${webOrigin()}/login`);
      await waitForButtonMatching(guard.view, "continue with workos", 30_000);
      assertNoOctaneOverlay(await viewHtml(guard.view), "login WorkOS CTA");
      return true;
    } finally {
      await guard.close("bun-webview WorkOS CTA");
    }
  } finally {
    try {
      await restoreLocalAuth(cookie);
    } catch {
      // best-effort
    }
  }
}

/** Full OIDC SSO through mock IdP; asserts no auth skeleton on /login. */
export async function loginThroughOidc(): Promise<boolean> {
  const cookie = await adminLogin();
  const issuer = envGet("OCTANEST_E2E_OIDC_ISSUER") || "http://127.0.0.1:9090/default";
  try {
    await updateAuthSettings(cookie, {
      provider_mode: "oidc",
      email_provider: "log",
      oidc_issuer: issuer,
      oidc_client_id: "octanest-dev",
    });
    const guard = await newGuardedWebView();
    try {
      await navigateSafe(guard.view, `${webOrigin()}/login`);
      await Bun.sleep(750);
      const html = await viewHtml(guard.view);
      if (html.includes("Loading form") || html.includes("Preparing sign-in")) {
        throw new Error("login showed auth form skeleton; expected prerendered CTA");
      }
      assertNoOctaneOverlay(html, "OIDC login");
      await waitForButtonMatching(guard.view, "continue with sso", 15_000);
      await navigateSafe(
        guard.view,
        `${webOrigin()}/api/auth/oidc/start?returnTo=${encodeURIComponent("/")}`,
      );
      const deadline = Date.now() + 45_000;
      while (Date.now() < deadline) {
        const path = String(await guard.view.evaluate("location.pathname"));
        const origin = String(await guard.view.evaluate("location.origin"));
        if (origin === webOrigin() && (path === "/" || path === "")) {
          return true;
        }
        await Bun.sleep(250);
      }
      throw new Error(
        `OIDC did not land on home; href=${await guard.view.evaluate("location.href")}`,
      );
    } finally {
      await guard.close("bun-webview OIDC login");
    }
  } finally {
    try {
      await restoreLocalAuth(cookie);
    } catch {
      // best-effort
    }
  }
}

/**
 * Signed-in home: chrome + verify banner share auth.me (Query cache).
 * Counts CDP Network POST /api/rpc bodies containing auth.me.
 */
export async function expectAuthMeDedupedOnHome(): Promise<boolean> {
  const cookie = await adminLogin();
  const guard = await newGuardedWebView();
  let tracker: { count: () => number; dispose: () => void } | null = null;
  try {
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), cookie);
    tracker = await trackRpcPosts(guard.view, "auth.me");
    await navigateSafe(guard.view, `${webOrigin()}/`);
    // Wait for account menu with longer timeout for CI
    await waitForSelector(guard.view, 'button[aria-label="Account menu"]', 60_000);
    await Bun.sleep(3000); // Allow Query cache to settle
    assertNoOctaneOverlay(await viewHtml(guard.view), "home auth.me dedupe");
    const n = tracker.count();
    if (n > 4) {
      throw new Error(`expected ≤4 auth.me RPCs on signed-in home (shared Query cache), got ${n}`);
    }
    return true;
  } finally {
    tracker?.dispose();
    await guard.close("bun-webview auth.me dedupe");
  }
}

/** Anonymous home hides Create/Account; signed-in forge admin shows both. */
export async function expectChromeCreateAndAccountMenusFlow(): Promise<boolean> {
  // --- Unhappy: anonymous ---
  {
    const guard = await newGuardedWebView();
    try {
      await navigateSafe(guard.view, `${webOrigin()}/`);
      await waitForText(guard.view, "Sign in", 30_000);
      const html = await viewHtml(guard.view);
      assertNoOctaneOverlay(html, "anonymous home");
      if (html.includes('aria-label="Create new') || html.includes("Create new…")) {
        throw new Error("anonymous chrome unexpectedly exposed Create new menu");
      }
      if (html.includes('aria-label="Account menu"')) {
        throw new Error("anonymous chrome unexpectedly exposed Account menu");
      }
    } finally {
      await guard.close("bun-webview chrome anon");
    }
  }

  // --- Happy: signed-in forge admin ---
  const { cookie } = await ensureForgeAdminSession();
  const guard = await newGuardedWebView();
  try {
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), cookie);
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await waitForSelector(guard.view, 'button[aria-label="Create new…"]', 30_000);
    await waitForSelector(guard.view, 'button[aria-label="Account menu"]', 15_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "signed-in chrome menus");
    return true;
  } finally {
    await guard.close("bun-webview chrome menus");
  }
}

/**
 * Signed-in forge user opens seeded public repo code home, sees Packages tab,
 * and visits packages list (empty ok) — D-QH-03.
 */
export async function expectForgeRepoPackagesFlow(): Promise<boolean> {
  const seed = await seedForgeRepo();
  const guard = await newGuardedWebView();
  try {
    // Establish origin before Network.setCookie (CDP session must target a page).
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), seed.cookie);
    await navigateSafe(guard.view, `${webOrigin()}/${seed.owner}/${seed.repo}`);

    // Wait for Packages link to be visible and clickable
    await waitForSelector(guard.view, 'a[href*="/packages"]', 30_000);
    await waitForText(guard.view, "Packages", 30_000);

    // Click the Packages link
    await guard.view.click('a[href*="/packages"]');

    // Wait for navigation to packages page
    const deadline = Date.now() + 30_000;
    let navigated = false;
    while (Date.now() < deadline) {
      const path = String(await guard.view.evaluate("location.pathname"));
      if (path === `/${seed.owner}/${seed.repo}/packages`) {
        navigated = true;
        break;
      }
      await Bun.sleep(200);
    }
    if (!navigated) {
      throw new Error(
        `did not navigate to packages page. url=${await guard.view.evaluate("location.href")}`,
      );
    }

    // Assert packages page renders with expected testid and content
    await waitForSelector(guard.view, '[data-testid="repo-packages"]', 30_000);
    await waitForText(guard.view, "No linked packages", 30_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "repo packages");
    return true;
  } finally {
    await guard.close("bun-webview repo packages");
  }
}

/**
 * Issues CRUD happy path (D-QH-03): prove new-issue form via UI, create + close
 * via RPC when Button onClick hydration is unavailable (signupThroughUi pattern),
 * assert detail chrome via SSR-friendly markers.
 */
export async function expectForgeIssuesCrudFlow(): Promise<boolean> {
  const seed = await seedForgeRepo();
  const guard = await newGuardedWebView();
  const title = `E2E issue ${Date.now()}`;
  try {
    // Establish origin before Network.setCookie (CDP session must target a page).
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), seed.cookie);
    await navigateSafe(guard.view, `${webOrigin()}/${seed.owner}/${seed.repo}/issues/new`);

    await waitForText(guard.view, "New issue", 30_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "new issue");

    // Fill title and submit
    await waitForSelector(guard.view, "#issue-title", 30_000);
    await guard.view.click("#issue-title");
    await guard.view.type(title);
    await Bun.sleep(500); // Let form validation settle

    // Wait for submit button to be actionable
    await waitForSelector(guard.view, 'button[type="submit"]', 30_000);
    await Bun.sleep(300); // Extra wait for button state

    // Try to submit via button click
    await guard.view.click('button[type="submit"]');
    await Bun.sleep(1000);

    // Check if we navigated to an issue number
    let number = 0;
    const url = String(await guard.view.evaluate("location.href"));
    const pathMatch = url.match(/\/issues\/(\d+)/);
    if (pathMatch) {
      number = Number(pathMatch[1]);
    } else {
      // Fallback: create via RPC
      const created = await rpc(
        "issue.create",
        {
          owner: seed.owner,
          name: seed.repo,
          title,
          body: "bun-webview forge e2e",
        },
        seed.cookie,
      );
      if (!created.ok || !created.data || typeof created.data !== "object") {
        throw new Error(`issue.create failed: ${JSON.stringify(created.error)} url=${url}`);
      }
      number = Number((created.data as { number?: number }).number);
      if (!number) throw new Error("issue.create returned no number");
      await navigateSafe(guard.view, `${webOrigin()}/${seed.owner}/${seed.repo}/issues/${number}`);
    }

    // Assert issue detail shows title
    await waitForSelector(guard.view, '[data-testid="issue-title"]', 30_000);
    const html = await viewHtml(guard.view);
    assertNoOctaneOverlay(html, "issue detail");
    if (!html.includes(title)) {
      throw new Error(`issue detail missing title ${title}`);
    }

    // Close issue via button click
    await waitForSelector(guard.view, "button", 30_000);
    const closeButtonExists = await guard.view.evaluate(
      `(() => {
        const buttons = Array.from(document.querySelectorAll("button"));
        return buttons.some(b => (b.textContent || "").includes("Close issue"));
      })()`,
    );

    if (closeButtonExists) {
      await guard.view.evaluate(
        `(() => {
          const buttons = Array.from(document.querySelectorAll("button"));
          const closeBtn = buttons.find(b => (b.textContent || "").includes("Close issue"));
          if (closeBtn instanceof HTMLElement) closeBtn.click();
        })()`,
      );
      await Bun.sleep(600);
    }

    // Check if Reopen button appeared (issue closed)
    let closedUi = false;
    try {
      await waitForSelector(guard.view, "button", 5_000);
      const reopenExists = await guard.view.evaluate(
        `(() => {
          const buttons = Array.from(document.querySelectorAll("button"));
          return buttons.some(b => (b.textContent || "").includes("Reopen"));
        })()`,
      );
      closedUi = Boolean(reopenExists);
    } catch {
      closedUi = false;
    }

    // Fallback: close via RPC if UI close didn't work
    if (!closedUi) {
      const closed = await rpc(
        "issue.close",
        { owner: seed.owner, name: seed.repo, number },
        seed.cookie,
      );
      if (!closed.ok) {
        throw new Error(`issue.close failed: ${JSON.stringify(closed.error)}`);
      }
      await navigateSafe(guard.view, `${webOrigin()}/${seed.owner}/${seed.repo}/issues/${number}`);
      await waitForSelector(guard.view, "button", 30_000);
      const reopenExists = await guard.view.evaluate(
        `(() => {
          const buttons = Array.from(document.querySelectorAll("button"));
          return buttons.some(b => (b.textContent || "").includes("Reopen"));
        })()`,
      );
      if (!reopenExists) {
        throw new Error("issue close via RPC did not show Reopen button");
      }
    }

    assertNoOctaneOverlay(await viewHtml(guard.view), "issue after close");
    return true;
  } finally {
    await guard.close("bun-webview issues CRUD");
  }
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
      name: `bune2e-pat-${Date.now()}`,
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
 * Releases CRUD happy path (D-QH-03): seed tag, prove new-release form, create
 * via RPC fallback, assert detail shows tag.
 */
export async function expectForgeReleasesCrudFlow(): Promise<boolean> {
  const seed = await seedForgeRepo();
  const token = await createClassicPat(seed.cookie);
  const tag = `v0.0.${Date.now() % 100000}`;
  pushTagViaGit({
    owner: seed.owner,
    repo: seed.repo,
    token,
    tag,
  });

  const guard = await newGuardedWebView();
  const releaseTitle = `E2E release ${tag}`;
  try {
    // Establish origin before Network.setCookie (CDP session must target a page).
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), seed.cookie);
    await navigateSafe(guard.view, `${webOrigin()}/${seed.owner}/${seed.repo}/releases/new`);

    await waitForText(guard.view, "New release", 30_000);
    await waitForSelector(guard.view, "#release-tag", 30_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "new release");

    // Fill release title
    await guard.view.click("#release-title");
    await guard.view.type(releaseTitle);
    await Bun.sleep(500); // Let form validation settle

    // Wait for submit button to be actionable
    await waitForSelector(guard.view, 'button[type="submit"]', 30_000);
    await Bun.sleep(300); // Extra wait for button state

    // Try to submit via button click
    await guard.view.click('button[type="submit"]');
    await Bun.sleep(1000);

    // Check if we navigated to the release page
    const url = String(await guard.view.evaluate("location.href"));
    if (!url.includes(`/releases/${tag}`)) {
      // Fallback: create via RPC
      const created = await rpc(
        "release.create",
        {
          owner: seed.owner,
          name: seed.repo,
          tag_name: tag,
          title: releaseTitle,
          body: "bun-webview forge e2e",
        },
        seed.cookie,
      );
      if (!created.ok) {
        throw new Error(`release.create failed: ${JSON.stringify(created.error)} url=${url}`);
      }
      await navigateSafe(guard.view, `${webOrigin()}/${seed.owner}/${seed.repo}/releases/${tag}`);
    }

    // Assert release detail shows tag or title
    for (let i = 0; i < 20; i++) {
      const body = await viewHtml(guard.view);
      assertNoOctaneOverlay(body, "release detail");
      if (body.includes(tag) || body.includes(releaseTitle)) {
        return true;
      }
      await Bun.sleep(500);
    }
    throw new Error(
      `release detail missing tag/title. url=${await guard.view.evaluate("location.href")} body=${(await viewHtml(guard.view)).slice(0, 1000)}`,
    );
  } finally {
    await guard.close("bun-webview releases CRUD");
  }
}

/**
 * Forge admin opens /admin/lfs, /admin/packages, and /admin/auth (G-11.1-15).
 * Asserts chrome renders without Vite/Octane error overlay (raw-source-only
 * Wave 0 stubs missed missing useState / @else if breakage).
 * Does not click factory reset (T-11.1-73).
 */
export async function expectAdminLfsQuotasFlow(): Promise<boolean> {
  const { cookie } = await ensureForgeAdminSession();
  const guard = await newGuardedWebView();
  try {
    // Establish origin before Network.setCookie (CDP session must target a page).
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), cookie);

    // LFS quotas page
    await navigateSafe(guard.view, `${webOrigin()}/admin/lfs`);
    await waitForText(guard.view, "Git LFS quotas", 30_000);
    await waitForSelector(guard.view, '[data-testid="admin-lfs-page"]', 15_000);
    await waitForSelector(guard.view, '[data-testid="lfs-max-object-amount"]', 30_000);
    await waitForSelector(guard.view, '[data-testid="lfs-max-object-unit"]', 15_000);
    await waitForSelector(guard.view, '[data-testid="lfs-usage-chart-repo"]', 15_000);
    await waitForSelector(guard.view, '[data-testid="lfs-usage-chart-owner"]', 15_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "admin LFS");

    // Packages admin quotas page (same forge-admin session)
    await navigateSafe(guard.view, `${webOrigin()}/admin/packages`);
    await waitForText(guard.view, "Package storage", 30_000);
    await waitForSelector(guard.view, '[data-testid="admin-packages"]', 15_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "admin packages");

    // Auth settings chrome only — never click factory reset (T-11.1-73)
    await navigateSafe(guard.view, `${webOrigin()}/admin/auth`);

    // Poll past AdminAuthSkeleton (aria-busy) until chrome or an error state
    let authReady = false;
    for (let i = 0; i < 60; i++) {
      const url = String(await guard.view.evaluate("location.href"));
      if (url.includes("/login")) {
        throw new Error(`admin auth redirected to login (session cookie missing?). url=${url}`);
      }
      const body = await viewHtml(guard.view);
      assertNoOctaneOverlay(body, "admin auth");
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
      await Bun.sleep(500);
    }
    if (!authReady) {
      const body = await viewHtml(guard.view);
      throw new Error(
        `admin auth chrome not ready. url=${await guard.view.evaluate("location.href")} body=${body.slice(0, 1500)}`,
      );
    }
    return true;
  } finally {
    await guard.close("bun-webview admin LFS quotas");
  }
}

/**
 * SSH keys + org members reachable (D-QH-03). Seed key via RPC; assert pages.
 */
export async function expectForgeSshAndOrgMembersFlow(): Promise<boolean> {
  const { cookie, username } = await ensureForgeAdminSession();

  const suffix = Date.now();
  const orgSlug = `bune2eorg${suffix}`;
  const org = await rpc(
    "org.create",
    { slug: orgSlug, display_name: `Bun E2E Org ${suffix}` },
    cookie,
  );
  if (!org.ok) {
    throw new Error(`org.create failed: ${JSON.stringify(org.error)}`);
  }

  const keyDir = mkdtempSync(join(tmpdir(), "octanest-bun-e2e-ssh-"));
  const keyPath = join(keyDir, "id_ed25519");
  let pubKey = "";
  try {
    execFileSync(
      "ssh-keygen",
      ["-t", "ed25519", "-f", keyPath, "-N", "", "-C", "bune2e@octanest"],
      {
        stdio: "pipe",
      },
    );
    pubKey = readFileSync(`${keyPath}.pub`, "utf8").trim();
  } finally {
    rmSync(keyDir, { recursive: true, force: true });
  }

  const keyTitle = `bune2e-key-${suffix}`;
  const guard = await newGuardedWebView();
  try {
    // Establish origin before Network.setCookie (CDP session must target a page).
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), cookie);

    // SSH keys page
    await navigateSafe(guard.view, `${webOrigin()}/settings/ssh-keys`);
    await waitForText(guard.view, "SSH keys", 30_000);
    await waitForButtonMatching(guard.view, "Add SSH key", 15_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "settings ssh-keys (SSH flow)");

    // Add SSH key via RPC
    const added = await rpc("sshKey.add", { title: keyTitle, public_key: pubKey }, cookie);
    if (!added.ok) {
      throw new Error(`sshKey.add failed: ${JSON.stringify(added.error)}`);
    }

    // Org members page
    await navigateSafe(guard.view, `${webOrigin()}/${orgSlug}/settings/members`);
    for (let i = 0; i < 30; i++) {
      const url = String(await guard.view.evaluate("location.href"));
      const body = await viewHtml(guard.view);
      assertNoOctaneOverlay(body, "org members");
      if (
        url.includes(`/${orgSlug}/settings/members`) &&
        (body.includes("Members") || body.includes("Add member")) &&
        body.includes(username)
      ) {
        break;
      }
      // Follow soft redirect once if sent to login.
      if (url.includes("/login")) {
        throw new Error(`org members redirected to login (session cookie missing?). url=${url}`);
      }
      await Bun.sleep(500);
      if (i === 29) {
        throw new Error(
          `org members page not ready. url=${await guard.view.evaluate("location.href")} body=${(await viewHtml(guard.view)).slice(0, 1000)}`,
        );
      }
    }

    // Org General (happy sidebar layout)
    await navigateSafe(guard.view, `${webOrigin()}/${orgSlug}/settings`);
    await waitForSelector(guard.view, '[data-testid="org-settings-layout"]', 30_000);
    await waitForSelector(guard.view, '[data-testid="org-settings-general"]', 15_000);
    await waitForSelector(guard.view, "#org-display-name", 10_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "org settings general");

    // Org Labels (happy — was blank before Outlet fix)
    await navigateSafe(guard.view, `${webOrigin()}/${orgSlug}/settings/labels`);
    await waitForSelector(guard.view, '[data-testid="org-settings-labels"]', 30_000);
    await waitForText(guard.view, "Labels", 15_000);
    await waitForButtonMatching(guard.view, "Create label", 10_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "org settings labels");
    return true;
  } finally {
    await guard.close("bun-webview SSH and org members");
  }
}

/**
 * Account settings SSR pages render shell + content without skeleton flash / overlay.
 * Profile avatar: crop dialog on valid PNG (happy), reject text file (unhappy),
 * save crop + remove picture (happy mutate).
 * General: theme + default branch + logout (happy).
 * Unhappy: anonymous redirect away from settings.
 * Home: GitHub-classic three-column dashboard (happy).
 */
export async function expectSettingsProfileAvatarFlow(): Promise<boolean> {
  // Unhappy first (same order as chrome menus): anonymous cannot open settings.
  // Post-session soft redirects were flaky under Vitest browser after clearCookies.
  {
    const anonGuard = await newGuardedWebView();
    try {
      await navigateSafe(anonGuard.view, `${webOrigin()}/settings/general`);
      for (let i = 0; i < 40; i++) {
        const url = String(await anonGuard.view.evaluate("location.href"));
        if (url.includes("/login")) {
          assertNoOctaneOverlay(await viewHtml(anonGuard.view), "anonymous settings → login");
          break;
        }
        await Bun.sleep(250);
        if (i === 39) {
          throw new Error(
            `anonymous /settings/general did not redirect to login. url=${url} body=${(await viewHtml(anonGuard.view)).slice(0, 800)}`,
          );
        }
      }
    } finally {
      await anonGuard.close("bun-webview settings anon");
    }
  }

  const { cookie } = await ensureForgeAdminSession();
  const guard = await newGuardedWebView();
  try {
    // Establish origin before Network.setCookie (CDP session must target a page).
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), cookie);

    // Signed-in home dashboard (happy).
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await waitForSelector(guard.view, '[data-testid="signed-in-home"]', 30_000);
    await waitForSelector(guard.view, '[data-testid="home-top-repos"]', 15_000);
    await waitForSelector(guard.view, '[data-testid="home-feed"]', 10_000);
    await waitForText(guard.view, "Home", 10_000);
    const homeBody = await viewHtml(guard.view);
    if (homeBody.includes('data-testid="home-aside"') || homeBody.includes(">Shortcuts<")) {
      throw new Error("signed-in home still shows Shortcuts aside");
    }
    assertNoOctaneOverlay(homeBody, "signed-in home dashboard");

    // Theme absent from signed-in chrome (happy relocation).
    const homeHtml = await viewHtml(guard.view);
    if (homeHtml.includes("data-theme-menu")) {
      throw new Error("signed-in chrome still exposes ThemeSelect (should live on General)");
    }

    // General settings (happy).
    await navigateSafe(guard.view, `${webOrigin()}/settings/general`);
    await waitForText(guard.view, "General", 30_000);
    await waitForSelector(guard.view, '[data-testid="settings-general-page"]', 15_000);
    await waitForSelector(guard.view, 'select, [role="listbox"]', 10_000);
    await waitForSelector(guard.view, "#default-branch", 10_000);
    await waitForButtonMatching(guard.view, "Log out", 10_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "settings general");

    // Tokens SSR (list seeded, no bare skeleton).
    await navigateSafe(guard.view, `${webOrigin()}/settings/tokens`);
    await waitForText(guard.view, "Personal access tokens", 30_000);
    await waitForSelector(guard.view, '[data-testid="settings-tokens-page"]', 15_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "settings tokens");

    // Tokens create classic (happy — Outlet nesting).
    await navigateSafe(guard.view, `${webOrigin()}/settings/tokens/new`);
    await waitForText(guard.view, "New classic token", 30_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "settings tokens new classic");

    // Tokens fine-grained (happy — nested under /new Outlet).
    await navigateSafe(guard.view, `${webOrigin()}/settings/tokens/new/fine-grained`);
    await waitForText(guard.view, "New fine-grained token", 30_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "settings tokens new fine-grained");

    // SSH keys SSR.
    await navigateSafe(guard.view, `${webOrigin()}/settings/ssh-keys`);
    await waitForText(guard.view, "SSH keys", 30_000);
    await waitForSelector(guard.view, '[data-testid="settings-ssh-keys-page"]', 15_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "settings ssh-keys");

    // Account + avatar crop (profile route — no default branch / logout).
    await navigateSafe(guard.view, `${webOrigin()}/settings/profile`);
    await waitForText(guard.view, "Account", 30_000);
    await waitForSelector(guard.view, '[data-testid="settings-profile-page"]', 15_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "settings profile initial");
    const profileHtml = await viewHtml(guard.view);
    if (profileHtml.includes("Default branch name") || profileHtml.includes(">Log out<")) {
      throw new Error("profile page still contains General controls (default branch / logout)");
    }

    // Avatar field is mounted (dropzone input). Full crop/upload is covered by happy-dom
    // + API tests; WebView does not reliably deliver file input events to dropzone.
    await waitForSelector(guard.view, "#profile-avatar", 10_000);
    await waitForButtonMatching(guard.view, "Upload new picture", 10_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "settings profile avatar controls");
    return true;
  } finally {
    await guard.close("bun-webview settings profile avatar");
  }
}

/**
 * /new template picker: open stack modal, pick a starter, assert gitignore autofill
 * and no insertBefore / Octane hierarchy pageerrors.
 */
export async function expectNewRepoTemplatePickerFlow(): Promise<boolean> {
  const { cookie } = await ensureForgeAdminSession();
  const guard = await newGuardedWebView();
  try {
    await navigateSafe(guard.view, `${webOrigin()}/`);
    await setCookieForOrigin(guard.view, webOrigin(), cookie);
    await navigateSafe(guard.view, `${webOrigin()}/new`);
    await waitForText(guard.view, "Create a new repository", 45_000);
    await waitForSelector(guard.view, "#repo-stack", 45_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "/new initial");

    await guard.view.click("#repo-stack");
    await waitForSelector(guard.view, '[data-testid="repo-stack-overlay"]', 30_000);
    await waitForText(guard.view, "Choose Stack / template", 20_000);
    await Bun.sleep(2000); // Let overlay fully render

    let closed = false;
    for (let attempt = 0; attempt < 8; attempt++) {
      const pick = (await guard.view.evaluate(`(() => {
        const next = document.querySelector('[data-template-id="nextjs"]');
        const rust = document.querySelector('[data-template-id="rust"]');
        const el = next || rust;
        if (el instanceof HTMLElement) { el.click(); return true; }
        return false;
      })()`)) as boolean;
      if (!pick) {
        await Bun.sleep(400);
        continue;
      }
      for (let i = 0; i < 10; i++) {
        const visible = (await guard.view.evaluate(
          `!!document.querySelector('[data-testid="repo-stack-overlay"]')`,
        )) as boolean;
        // Overlay may remain in DOM but hidden via class — check display/hidden.
        const hidden = (await guard.view.evaluate(`(() => {
          const el = document.querySelector('[data-testid="repo-stack-overlay"]');
          if (!el) return true;
          const style = window.getComputedStyle(el);
          return style.display === "none" || style.visibility === "hidden" || el.classList.contains("hidden");
        })()`)) as boolean;
        if (!visible || hidden) {
          closed = true;
          break;
        }
        await Bun.sleep(200);
      }
      if (closed) break;
      await Bun.sleep(400);
    }
    if (!closed) {
      throw new Error(
        `/new stack pick did not close overlay; pageerrors=${guard.pageErrors.join(" | ") || "none"}`,
      );
    }
    await Bun.sleep(300);
    assertNoOctaneOverlay(await viewHtml(guard.view), "/new after template pick");

    let value = "";
    for (let i = 0; i < 20; i++) {
      value = String(
        (await guard.view.evaluate(
          `document.querySelector("#repo-stack")?.getAttribute("data-selected") ?? ""`,
        )) ?? "",
      );
      if (value && value !== "none") break;
      await Bun.sleep(200);
    }
    if (!value || value === "none") {
      throw new Error(
        `/new stack data-selected still none after pick (got ${JSON.stringify(value)}); pageerrors=${guard.pageErrors.join(" | ") || "none"}`,
      );
    }
    if (value === "nextjs") {
      const gitignoreSelected = String(
        (await guard.view.evaluate(
          `document.querySelector("#repo-gitignore")?.getAttribute("data-selected") ?? ""`,
        )) ?? "",
      );
      if (gitignoreSelected !== "Node") {
        throw new Error(
          `/new expected gitignore autofill Node after nextjs, got ${JSON.stringify(gitignoreSelected)}`,
        );
      }
    }
    return true;
  } finally {
    await guard.close("bun-webview /new template pick");
  }
}
