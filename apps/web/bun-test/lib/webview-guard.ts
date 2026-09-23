/**
 * Bun.WebView page-error guard (Playwright `newGuardedPage` parity for issue #37 PoC).
 *
 * Isolation:
 * - `dataStore: "ephemeral"` always
 * - Chrome spawn mode `url: false` (never attach to a desktop DevToolsActivePort)
 * - Force `backend: "chrome"` for CI/macOS parity
 * - Collect CDP `Runtime.exceptionThrown` as pageerrors (DOM races + other)
 */
import { DOM_RACE_RE } from "../../src/test/dom-errors.ts";

export type GuardedWebView = {
  view: Bun.WebView;
  pageErrors: string[];
  assertNoPageErrors: (label?: string) => void;
  /** Close first, then assert — matches Playwright guard teardown order. */
  close: (label?: string) => Promise<void>;
};

export type CreateGuardedWebViewOptions = {
  width?: number;
  height?: number;
  /** Optional Chrome binary (CI: Playwright chrome-headless-shell via BUN_CHROME_PATH). */
  chromePath?: string;
};

function chromeBackend(chromePath?: string): {
  type: "chrome";
  url: false;
  path?: string;
  argv?: string[];
} {
  const path = chromePath || process.env.BUN_CHROME_PATH || undefined;
  const argv = [
    "--no-sandbox",
    "--disable-setuid-sandbox",
    "--disable-dev-shm-usage",
    "--disable-gpu",
    "--remote-debugging-port=0",
  ];
  if (path) {
    return { type: "chrome", url: false, path, argv };
  }
  return { type: "chrome", url: false, argv };
}

export async function newGuardedWebView(
  opts: CreateGuardedWebViewOptions = {},
): Promise<GuardedWebView> {
  const pageErrors: string[] = [];
  const view = new Bun.WebView({
    width: opts.width ?? 1280,
    height: opts.height ?? 720,
    headless: true,
    dataStore: "ephemeral",
    backend: chromeBackend(opts.chromePath),
    // Mirror Playwright pageerror: only uncaught exceptions (via CDP below).
    // Do not treat Octane/devtools console warnings (e.g. invalid DOM prop
    // `autocomplete`) as failures — this app is Octane (`.tsrx`), not React.
  });

  // Establish CDP session with a blank document, then enable Runtime exceptions.
  await view.navigate("about:blank");
  try {
    await view.cdp("Runtime.enable");
  } catch (e) {
    await view.close();
    throw new Error(`Runtime.enable failed (need Chrome CDP): ${String(e)}`, { cause: e });
  }

  view.addEventListener("Runtime.exceptionThrown", ((event: Event) => {
    const data = (event as MessageEvent).data as {
      exceptionDetails?: { text?: string; exception?: { description?: string; value?: string } };
    };
    const details = data?.exceptionDetails;
    const msg =
      details?.exception?.description ||
      details?.exception?.value ||
      details?.text ||
      "unknown page exception";
    pageErrors.push(String(msg));
  }) as EventListener);

  const assertNoPageErrors = (label = "page") => {
    const races = pageErrors.filter((m) => DOM_RACE_RE.test(m));
    if (races.length > 0) {
      throw new Error(`${label} DOM race pageerror: ${races.join(" | ")}`);
    }
    if (pageErrors.length > 0) {
      throw new Error(`${label} unexpected pageerror: ${pageErrors.join(" | ")}`);
    }
  };

  return {
    view,
    pageErrors,
    assertNoPageErrors,
    async close(label = "page") {
      try {
        await view.close();
      } finally {
        assertNoPageErrors(label);
      }
    },
  };
}

export function assertNoOctaneOverlay(html: string, label: string): void {
  if (
    html.includes("vite-error-overlay") ||
    html.includes("Something went wrong!") ||
    /is not defined|ReferenceError|Octane error|@else if/i.test(html)
  ) {
    throw new Error(`${label} showed Vite/Octane render error. body=${html.slice(0, 1200)}`);
  }
}

export async function viewHtml(view: Bun.WebView): Promise<string> {
  return String(await view.evaluate("document.documentElement.outerHTML"));
}

export async function waitForText(
  view: Bun.WebView,
  text: string,
  timeoutMs = 30_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const found = await view.evaluate(
      `(() => document.body && document.body.innerText.includes(${JSON.stringify(text)}))()`,
    );
    if (found) return;
    await Bun.sleep(150);
  }
  throw new Error(`timeout waiting for text ${JSON.stringify(text)}`);
}

export async function waitForSelector(
  view: Bun.WebView,
  selector: string,
  timeoutMs = 30_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const found = await view.evaluate(
      `(() => !!document.querySelector(${JSON.stringify(selector)}))()`,
    );
    if (found) return;
    await Bun.sleep(150);
  }
  const url = await view.evaluate("location.href").catch(() => "?");
  const title = await view.evaluate("document.title").catch(() => "?");
  const snippet = await view
    .evaluate("document.body ? document.body.innerText.slice(0, 400) : ''")
    .catch(() => "");
  throw new Error(
    `timeout waiting for selector ${JSON.stringify(selector)} url=${url} title=${title} body=${JSON.stringify(snippet)}`,
  );
}

/** Wait until a button/link whose visible text matches `re` is in the DOM. */
export async function waitForButtonMatching(
  view: Bun.WebView,
  reSource: string,
  timeoutMs = 30_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const found = await view.evaluate(
      `(() => {
        const re = new RegExp(${JSON.stringify(reSource)}, "i");
        const nodes = Array.from(document.querySelectorAll("button, a[role='button'], a"));
        return nodes.some((el) => re.test((el.textContent || "").trim()));
      })()`,
    );
    if (found) return;
    await Bun.sleep(150);
  }
  const snippet = await view
    .evaluate("document.body ? document.body.innerText.slice(0, 800) : ''")
    .catch(() => "");
  throw new Error(
    `timeout waiting for button matching /${reSource}/i body=${JSON.stringify(snippet)}`,
  );
}

/** Enable CDP Network and count POST /api/rpc bodies that mention a procedure. */
export async function trackRpcPosts(
  view: Bun.WebView,
  procedure: string,
): Promise<{ count: () => number; dispose: () => void }> {
  await view.cdp("Network.enable");
  const bodies: string[] = [];
  const needleA = `"${procedure}"`;
  const needleB = `"procedure":"${procedure}"`;
  const handler = ((event: Event) => {
    const data = (event as MessageEvent).data as {
      request?: { method?: string; url?: string; postData?: string };
    };
    const req = data?.request;
    if (!req || req.method !== "POST") return;
    if (!req.url?.includes("/api/rpc")) return;
    const body = req.postData ?? "";
    if (body.includes(needleA) || body.includes(needleB)) {
      bodies.push(body);
    }
  }) as EventListener;
  view.addEventListener("Network.requestWillBeSent", handler);
  return {
    count: () => bodies.length,
    dispose: () => {
      try {
        view.removeEventListener("Network.requestWillBeSent", handler);
      } catch {
        // ignore
      }
    },
  };
}

/** Navigate and tolerate mid-flight target swaps (Vite HMR / redirects). */
export async function navigateSafe(view: Bun.WebView, url: string): Promise<void> {
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      await view.navigate(url);
      return;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (!/navigated or closed|Target closed|session/i.test(msg) || attempt === 2) {
        throw e;
      }
      await Bun.sleep(500);
    }
  }
}

export async function setCookieForOrigin(
  view: Bun.WebView,
  origin: string,
  cookieHeader: string,
): Promise<void> {
  const eq = cookieHeader.indexOf("=");
  const name = eq >= 0 ? cookieHeader.slice(0, eq) : "octanest_session";
  const value = eq >= 0 ? cookieHeader.slice(eq + 1) : cookieHeader;
  const url = new URL(origin);
  await view.cdp("Network.enable");
  await view.cdp("Network.setCookie", {
    name,
    value,
    domain: url.hostname,
    path: "/",
    url: origin,
  });
}
