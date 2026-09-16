import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const dir = dirname(fileURLToPath(import.meta.url));

describe("fork confirm route (SOC-04)", () => {
  it("defines ForkConfirmPage and repo.fork call", () => {
    const src = readFileSync(join(dir, "$owner.$repo.fork.tsrx"), "utf8");
    expect(src).toMatch(/export function ForkConfirmPage/);
    expect(src).toMatch(/createFileRoute\("\/\$owner\/\$repo\/fork"\)/);
    expect(src).toMatch(/apiClient\.repo\.fork/);
  });
});
