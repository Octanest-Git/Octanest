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

/** Wait until an element is actionable (visible, enabled, and stable). */
export async function waitForActionable(
  view: Bun.WebView,
  selector: string,
  timeoutMs = 30_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const actionable = await view.evaluate(
      `(() => {
        const el = document.querySelector(${JSON.stringify(selector)});
        if (!el) return false;
        const rect = el.getBoundingClientRect();
        const style = window.getComputedStyle(el);
        
        // Check visibility and dimensions
        if (rect.width <= 0 || rect.height <= 0) return false;
        if (style.display === 'none' || style.visibility === 'hidden') return false;
        if (parseFloat(style.opacity) === 0) return false;
        
        // Check if element is disabled (for form controls)
        if (el.disabled === true) return false;
        if (el.classList.contains('disabled')) return false;
        if (el.hasAttribute('disabled')) return false;
        if (el.getAttribute('aria-disabled') === 'true') return false;
        
        // Check if element is inside a disabled fieldset
        if (el.closest('fieldset[disabled]')) return false;
        
        return true;
      })()`,
    );
    if (actionable) return;
    await Bun.sleep(150);
  }

  // Enhanced diagnostics on timeout
  const diagnostics = await view
    .evaluate(
      `(() => {
      const el = document.querySelector(${JSON.stringify(selector)});
      if (!el) return { exists: false };
      const rect = el.getBoundingClientRect();
      const style = window.getComputedStyle(el);
      return {
        exists: true,
        tagName: el.tagName,
        type: el.type || null,
        disabled: el.disabled || false,
        classList: Array.from(el.classList),
        rect: { width: rect.width, height: rect.height, x: rect.x, y: rect.y },
        style: { display: style.display, visibility: style.visibility, opacity: style.opacity },
        formValid: el.form ? el.form.checkValidity() : null,
        formErrors: el.form ? Array.from(el.form.querySelectorAll(':invalid')).map(e => e.name || e.id || e.tagName) : [],
        textContent: el.textContent?.trim() || '',
        outerHTML: el.outerHTML.slice(0, 200)
      };
    })()`,
    )
    .catch(() => ({ error: "diagnostic failed" }));

  const url = await view.evaluate("location.href").catch(() => "?");
  const title = await view.evaluate("document.title").catch(() => "?");
  const snippet = await view
    .evaluate("document.body ? document.body.innerText.slice(0, 400) : ''")
    .catch(() => "");

  throw new Error(
    `timeout waiting for actionable ${JSON.stringify(selector)} url=${url} title=${title} diagnostics=${JSON.stringify(diagnostics)} body=${JSON.stringify(snippet)}`,
  );
}

/** Wait until a form is ready for submission (no validation errors, not loading). */
export async function waitForFormReady(
  view: Bun.WebView,
  formSelector = "form",
  timeoutMs = 30_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const ready = await view.evaluate(
      `(() => {
        const form = document.querySelector(${JSON.stringify(formSelector)});
        if (!form) return false;
        
        // Check for loading indicators
        const loading = document.querySelector('[data-loading="true"], .loading, .spinner, [aria-busy="true"]');
        if (loading) return false;
        
        // Check form validity
        if (form.checkValidity && !form.checkValidity()) return false;
        
        // Check for error messages
        const errors = document.querySelector('.error, [role="alert"], .field-error');
        if (errors && errors.textContent?.trim()) return false;
        
        return true;
      })()`,
    );
    if (ready) return;
    await Bun.sleep(200);
  }

  const diagnostics = await view
    .evaluate(
      `(() => {
      const form = document.querySelector(${JSON.stringify(formSelector)});
      const loading = document.querySelector('[data-loading="true"], .loading, .spinner, [aria-busy="true"]');
      const errors = Array.from(document.querySelectorAll('.error, [role="alert"], .field-error'));
      const invalidFields = form ? Array.from(form.querySelectorAll(':invalid')) : [];
      
      return {
        formExists: !!form,
        formValid: form ? form.checkValidity() : null,
        hasLoading: !!loading,
        loadingText: loading?.textContent?.trim() || '',
        errorCount: errors.length,
        errorTexts: errors.map(e => e.textContent?.trim()).filter(Boolean),
        invalidFieldCount: invalidFields.length,
        invalidFields: invalidFields.map(e => ({ name: e.name, id: e.id, tagName: e.tagName, validationMessage: e.validationMessage }))
      };
    })()`,
    )
    .catch(() => ({ error: "form diagnostic failed" }));

  throw new Error(
    `timeout waiting for form ready ${JSON.stringify(formSelector)} diagnostics=${JSON.stringify(diagnostics)}`,
  );
}

/** Submit a form using multiple strategies (click, Enter, form.submit()). */
export async function submitForm(
  view: Bun.WebView,
  submitSelector = 'button[type="submit"]',
  formSelector = "form",
): Promise<void> {
  // Strategy 1: Try clicking the submit button
  try {
    await waitForActionable(view, submitSelector, 10_000);
    await view.click(submitSelector);
    return;
  } catch (clickError) {
    // Strategy 2: Try pressing Enter on the form
    try {
      await view.evaluate(
        `(() => {
          const form = document.querySelector(${JSON.stringify(formSelector)});
          const submitBtn = document.querySelector(${JSON.stringify(submitSelector)});
          if (submitBtn) {
            submitBtn.focus();
            submitBtn.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
            return true;
          }
          return false;
        })()`,
      );
      return;
    } catch (enterError) {
      // Strategy 3: Direct form submission
      await view.evaluate(
        `(() => {
          const form = document.querySelector(${JSON.stringify(formSelector)});
          if (form) form.submit();
        })()`,
      );
    }
  }
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
