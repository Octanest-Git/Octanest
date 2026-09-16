import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

/**
 * SOC-01 / SOC-04 Wave 0 — RepoChrome Star + Fork affordances.
 * Turns green in 21-02 (Star) and 21-06 (Fork).
 */

const chromePath = join(dirname(fileURLToPath(import.meta.url)), "repo-chrome.tsrx");

describe("repo-chrome social (Wave 0)", () => {
  it("includes Star affordance and star_count display", () => {
    const src = readFileSync(chromePath, "utf8");
    expect(src).toMatch(/[Ss]tar/);
    expect(src).toMatch(/star_count|starCount/);
  });

  it("includes Fork affordance", () => {
    const src = readFileSync(chromePath, "utf8");
    expect(src).toMatch(/[Ff]ork/);
  });
});
