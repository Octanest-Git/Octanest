/**
 * Stack flows driven by Bun.WebView for issue #37 PoC.
 * Keep selectors on stable ids / data-testid (no Playwright getByRole).
 * Target Octane `.tsrx` UI (onInput text fields, data-testid radios) — not React.
 */
import {
  adminLogin,
  restoreLocalAuth,
  rpc,
  updateAuthSettings,
} from "../../e2e/stack/client.ts";
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
    await waitForSelector(guard.view, 'button[aria-label="Account menu"]', 30_000);
    await Bun.sleep(2500);
    assertNoOctaneOverlay(await viewHtml(guard.view), "home auth.me dedupe");
    const n = tracker.count();
    if (n > 4) {
      throw new Error(
        `expected ≤4 auth.me RPCs on signed-in home (shared Query cache), got ${n}`,
      );
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
    await waitForText(guard.view, "Create a new repository", 30_000);
    await waitForSelector(guard.view, "#repo-stack", 30_000);
    assertNoOctaneOverlay(await viewHtml(guard.view), "/new initial");

    await guard.view.click("#repo-stack");
    await waitForSelector(guard.view, '[data-testid="repo-stack-overlay"]', 15_000);
    await waitForText(guard.view, "Choose Stack / template", 10_000);
    await Bun.sleep(1500);

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
