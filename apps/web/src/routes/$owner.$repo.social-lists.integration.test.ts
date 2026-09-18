import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const dir = dirname(fileURLToPath(import.meta.url));

describe("repo social list routes", () => {
  it("wires stargazers page with Write+ gate and list SSR", () => {
    const src = readFileSync(join(dir, "$owner.$repo.stargazers.tsrx"), "utf8");
    expect(src).toMatch(/createFileRoute\("\/\$owner\/\$repo\/stargazers"\)/);
    expect(src).toMatch(/export function RepoStargazersPage/);
    expect(src).toMatch(/fetchRepoStargazers/);
    expect(src).toMatch(/can_write/);
    expect(src).toMatch(/Find a stargazer/);
  });

  it("wires watchers page with list SSR and search", () => {
    const src = readFileSync(join(dir, "$owner.$repo.watchers.tsrx"), "utf8");
    expect(src).toMatch(/createFileRoute\("\/\$owner\/\$repo\/watchers"\)/);
    expect(src).toMatch(/export function RepoWatchersPage/);
    expect(src).toMatch(/fetchRepoWatchers/);
    expect(src).toMatch(/Find a watcher/);
  });

  it("wires forks page with sort + search", () => {
    const src = readFileSync(join(dir, "$owner.$repo.forks.tsrx"), "utf8");
    expect(src).toMatch(/createFileRoute\("\/\$owner\/\$repo\/forks"\)/);
    expect(src).toMatch(/export function RepoForksPage/);
    expect(src).toMatch(/fetchRepoForks/);
    expect(src).toMatch(/parseForksSort/);
    expect(src).toMatch(/Find a fork/);
  });
});
