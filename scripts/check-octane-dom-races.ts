#!/usr/bin/env bun
/**
 * Heuristic scan: after </RadioGroup>, flag nearby @if … @else sibling swaps.
 * Those pairs often race Base UI DOM updates (insertBefore / HierarchyRequestError).
 *
 * Prefer keeping both panels mounted (`hidden` class) — see .agents/skills/octane/SKILL.md.
 *
 * Opt out a site with: // octane-dom-race-ok
 * on the same line as the `@else` (or the preceding `@if`).
 */
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const ROOT = join(import.meta.dir, "..", "apps", "web", "src");
const WINDOW = 50;

type Hit = { file: string; line: number; detail: string };

function walk(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) walk(p, out);
    else if (name.endsWith(".tsrx")) out.push(p);
  }
  return out;
}

function scan(file: string): Hit[] {
  const rel = relative(join(import.meta.dir, ".."), file);
  const lines = readFileSync(file, "utf8").split(/\r?\n/);
  const hits: Hit[] = [];

  for (let i = 0; i < lines.length; i++) {
    if (!lines[i]!.includes("</RadioGroup>")) continue;

    const end = Math.min(lines.length, i + 1 + WINDOW);
    let ifLine = -1;
    let ifIndent = -1;
    for (let j = i + 1; j < end; j++) {
      const line = lines[j]!;
      if (line.includes("</RadioGroup>")) break;

      const ifMatch = line.match(/^(\s*)@if\s*\(/);
      if (ifMatch && ifLine < 0) {
        ifLine = j;
        ifIndent = ifMatch[1]!.length;
        continue;
      }
      if (ifLine < 0) continue;

      const elseMatch = line.match(/^(\s*)\}\s*@else\s*\{/);
      if (elseMatch && elseMatch[1]!.length === ifIndent) {
        const region = lines.slice(ifLine, j + 1).join("\n");
        if (/octane-dom-race-ok/.test(region)) break;
        hits.push({
          file: rel,
          line: ifLine + 1,
          detail:
            "`</RadioGroup>` followed by sibling `@if`/`@else` within " +
            `${WINDOW} lines — prefer CSS hidden panels or // octane-dom-race-ok`,
        });
        break;
      }
    }
  }
  return hits;
}

const files = walk(ROOT);
const hits = files.flatMap(scan);

if (hits.length === 0) {
  console.log("check-octane-dom-races: ok");
  process.exit(0);
}

console.error("check-octane-dom-races: potential RadioGroup + @if/@else sibling swaps:\n");
for (const h of hits) {
  console.error(`  ${h.file}:${h.line}: ${h.detail}`);
}
console.error(
  "\nSee .agents/skills/octane/SKILL.md (insertBefore failure mode) and apps/web/src/test/dom-errors.ts.",
);
process.exit(1);
