/**
 * bun:test preload for dual-run lib integration (happy-dom).
 * Does not stub `@octanejs/tanstack-query` — that package ships `.tsrx` and
 * session-cache stays on Vitest. Theme + other pure-DOM lib tests dual-run here.
 */
import { GlobalRegistrator } from "@happy-dom/global-registrator";
import { afterAll, afterEach, beforeEach, expect } from "bun:test";
import * as matchers from "@testing-library/jest-dom/matchers";
import {
  consumeDomRaceAllowlist,
  trackDomErrors,
  type DomErrorTracker,
} from "../src/test/dom-errors.ts";

expect.extend(matchers);

if (!(globalThis as { __octanestHappyDom?: boolean }).__octanestHappyDom) {
  GlobalRegistrator.register();
  (globalThis as { __octanestHappyDom?: boolean }).__octanestHappyDom = true;

  // happy-dom's document.cookie is often a no-op; mirror a simple jar for theme tests.
  let cookieJar = "";
  Object.defineProperty(document, "cookie", {
    configurable: true,
    get() {
      return cookieJar;
    },
    set(value: string) {
      const part = String(value).split(";")[0] ?? "";
      const eq = part.indexOf("=");
      if (eq < 0) return;
      const name = part.slice(0, eq).trim();
      const rest = cookieJar.split("; ").filter((c) => c && !c.startsWith(`${name}=`));
      rest.push(part.trim());
      cookieJar = rest.join("; ");
    },
  });
}

let suiteTracker: DomErrorTracker | null = null;

beforeEach(() => {
  suiteTracker?.dispose();
  suiteTracker = trackDomErrors();
});

afterEach(() => {
  const tracker = suiteTracker;
  suiteTracker = null;
  if (!tracker) return;
  try {
    if (!consumeDomRaceAllowlist()) {
      tracker.expectNoDomRaces();
    }
  } finally {
    tracker.dispose();
  }
});

afterAll(() => {
  try {
    Bun.WebView?.closeAll?.();
  } catch {
    // ignore
  }
});
