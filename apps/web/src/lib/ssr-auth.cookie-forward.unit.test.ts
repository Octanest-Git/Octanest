import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

/**
 * T-06-11: SSR Cookie-forward contract (createSsrClient is module-private —
 * assert the secure behavior stays wired without exporting internals).
 */
describe("SSR Cookie-forward (T-06-11)", () => {
  const src = readFileSync(
    path.join(path.dirname(fileURLToPath(import.meta.url)), "ssr-auth.ts"),
    "utf8",
  );

  it("forwards incoming Cookie on SSR RPC fetch when cookie is present", () => {
    expect(src).toMatch(/if\s*\(\s*cookie\s*\)/);
    expect(src).toMatch(/headers\.set\(\s*["']cookie["']\s*,\s*cookie\s*\)/);
  });

  it("reads Cookie from the incoming SSR request header", () => {
    expect(src).toMatch(/getRequestHeader\(\s*["']cookie["']\s*\)/);
  });

  it("never logs cookie values", () => {
    expect(src).not.toMatch(/console\.(log|debug|info|warn|error)\([^)]*cookie/i);
  });

  it("targets API origin for SSR client (not browser origin)", () => {
    expect(src).toMatch(/OXIDEAN_API_ORIGIN/);
    expect(src).toMatch(/createSsrClient\s*\(\s*incomingCookie\s*\(\s*\)\s*\)/);
  });
});
