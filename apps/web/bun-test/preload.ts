/**
 * bun:test preload for the issue #37 PoC.
 * Keeps globals minimal — unit tests need no DOM; browser tests create WebViews explicitly.
 */
import { afterAll } from "bun:test";

afterAll(() => {
  try {
    if (typeof Bun !== "undefined" && Bun.WebView?.closeAll) {
      Bun.WebView.closeAll();
    }
  } catch {
    // ignore — Chrome may already be gone
  }
});
