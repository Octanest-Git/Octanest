/**
 * Vite/Rolldown hard-fails when a value import names a type-only export
 * (`MISSING_EXPORT`; see rolldown#9197). Some published packages still do this.
 *
 * This plugin rewrites those imports to `import type` inside matched modules.
 * Configure `include` / `externalTypeOnly` / `tsrxShimBasenames` per package.
 */
import fs from "node:fs";
import path from "node:path";
import type { Plugin } from "vite";

/** Named imports (single- or multi-line). */
const IMPORT_RE = /import\s+(?!type\b)\{([\s\S]*?)\}\s+from\s+(['"])([^'"]+)\2\s*;/g;
const EXPORT_TYPE_RE = /export\s+type\s+(?:\{([^}]+)\}|(\w+))|export\s+interface\s+(\w+)/g;

export type FixTypeOnlyImportsOptions = {
  /**
   * Absolute module ids to rewrite (typically `node_modules/.../pkg/src/...`).
   * Bun may nest packages as `@scope+name@version/...`.
   */
  include: RegExp | readonly RegExp[];
  /**
   * External package specs → bindings that exist only as types (e.g. in `.d.ts`
   * but not in the ESM runtime entry).
   */
  externalTypeOnly?: Readonly<Record<string, readonly string[]>>;
  /**
   * Basename shims like `Bar.ts` that `export type *` from `Bar.tsrx` — resolve
   * type exports (and rewrite type import specs) to the `.tsrx` sibling.
   */
  tsrxShimBasenames?: readonly string[];
};

/** Preset for `@octanejs/recharts@0.1.50` (+ `victory-vendor` type-only symbols). */
export const RECHARTS_TYPE_ONLY_IMPORT_FIX: FixTypeOnlyImportsOptions = {
  include:
    /[/\\]@octanejs(?:\+|\/)recharts(?:@[^/\\]+)?[/\\](?:node_modules[/\\]@octanejs[/\\]recharts[/\\])?src[/\\]/,
  externalTypeOnly: {
    "victory-vendor/d3-shape": ["Series", "SeriesPoint", "SymbolType"],
  },
  tsrxShimBasenames: ["Bar", "Line"],
};

function stripViteNullPrefix(id: string): string {
  const nul = String.fromCharCode(0);
  const cut = id.indexOf(nul);
  return cut === -1 ? id : id.slice(0, cut);
}

function matchesInclude(id: string, include: FixTypeOnlyImportsOptions["include"]): boolean {
  const patterns = Array.isArray(include) ? include : [include];
  return patterns.some((re) => re.test(id));
}

function resolveImport(importer: string, spec: string): string | null {
  if (!spec.startsWith(".")) return null;
  const base = path.resolve(path.dirname(importer), spec);
  const candidates = [
    base,
    `${base}.ts`,
    `${base}.tsrx`,
    path.join(base, "index.ts"),
    path.join(base, "index.tsrx"),
  ];
  for (const candidate of candidates) {
    if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) {
      return candidate;
    }
  }
  return null;
}

function collectTypeExports(filePath: string, cache: Map<string, Set<string> | null>): Set<string> {
  const cached = cache.get(filePath);
  if (cached) return cached;
  if (cached === null) return new Set();
  if (!fs.existsSync(filePath)) {
    cache.set(filePath, null);
    return new Set();
  }
  const text = fs.readFileSync(filePath, "utf8");
  const names = new Set<string>();
  for (const match of text.matchAll(EXPORT_TYPE_RE)) {
    if (match[1]) {
      for (const part of match[1].split(",")) {
        const trimmed = part.trim();
        if (!trimmed) continue;
        const [left, right] = trimmed.split(/\s+as\s+/);
        names.add((right ?? left).trim());
        names.add(left.trim());
      }
    }
    if (match[2]) names.add(match[2]);
    if (match[3]) names.add(match[3]);
  }
  const starRe = /export\s+type\s+\*\s+from\s+['"]([^'"]+)['"]/g;
  for (const match of text.matchAll(starRe)) {
    const target = resolveImport(filePath, match[1]);
    if (!target) continue;
    for (const name of collectTypeExports(target, cache)) names.add(name);
  }
  cache.set(filePath, names);
  return names;
}

function parseBinding(part: string): { local: string; orig: string; isType: boolean } | null {
  const trimmed = part.trim();
  if (!trimmed) return null;
  const isType = trimmed.startsWith("type ");
  const body = isType ? trimmed.slice(5).trim() : trimmed;
  const [local] = body.split(/\s+as\s+/);
  return { local: local.trim(), orig: body, isType };
}

function isTsrxShim(resolved: string, basenames: readonly string[]): boolean {
  for (const name of basenames) {
    if (resolved.endsWith(`${path.sep}${name}.ts`)) return true;
  }
  return false;
}

function rewriteTypeOnlyImports(
  code: string,
  id: string,
  options: FixTypeOnlyImportsOptions,
  cache: Map<string, Set<string> | null>,
): string | null {
  const external = options.externalTypeOnly ?? {};
  const shims = options.tsrxShimBasenames ?? [];
  let changed = false;

  const next = code.replace(IMPORT_RE, (full, namesRaw: string, quote: string, spec: string) => {
    let typeNames: Set<string>;
    let importSpec = spec;

    if (Object.prototype.hasOwnProperty.call(external, spec)) {
      typeNames = new Set(external[spec]);
    } else if (spec.startsWith(".")) {
      let resolved = resolveImport(id, spec);
      if (resolved && isTsrxShim(resolved, shims)) {
        const tsrx = `${resolved}rx`;
        if (fs.existsSync(tsrx)) {
          resolved = tsrx;
          if (!spec.endsWith(".tsrx")) importSpec = `${spec}.tsrx`;
        }
      }
      if (!resolved) return full;
      typeNames = collectTypeExports(resolved, cache);
      if (typeNames.size === 0) return full;
    } else {
      return full;
    }

    const bindings = namesRaw
      .split(",")
      .map(parseBinding)
      .filter((b): b is NonNullable<typeof b> => b !== null);

    const typeParts: string[] = [];
    const valueParts: string[] = [];
    let moved = false;
    for (const binding of bindings) {
      if (binding.isType || typeNames.has(binding.local)) {
        typeParts.push(binding.orig);
        if (!binding.isType) moved = true;
      } else {
        valueParts.push(binding.orig);
      }
    }
    if (!moved) return full;

    changed = true;
    const out: string[] = [];
    if (valueParts.length > 0) {
      out.push(`import { ${valueParts.join(", ")} } from ${quote}${spec}${quote};`);
    }
    out.push(`import type { ${typeParts.join(", ")} } from ${quote}${importSpec}${quote};`);
    return out.join("\n");
  });

  return changed ? next : null;
}

/**
 * Rewrite value imports of type-only symbols to `import type` for Rolldown.
 * Pass one or more package presets (e.g. `RECHARTS_TYPE_ONLY_IMPORT_FIX`).
 */
export function fixTypeOnlyImports(...presets: FixTypeOnlyImportsOptions[]): Plugin {
  if (presets.length === 0) {
    throw new Error("fixTypeOnlyImports: pass at least one options preset");
  }
  const cache = new Map<string, Set<string> | null>();

  return {
    name: "fix-type-only-imports",
    enforce: "pre",
    transform(code, id) {
      const cleanId = stripViteNullPrefix(id).split("?")[0] ?? id;
      if (!cleanId.endsWith(".ts") && !cleanId.endsWith(".tsrx")) return null;

      let current = code;
      let any = false;
      for (const options of presets) {
        if (!matchesInclude(cleanId, options.include)) continue;
        const rewritten = rewriteTypeOnlyImports(current, cleanId, options, cache);
        if (rewritten) {
          current = rewritten;
          any = true;
        }
      }
      if (!any) return null;
      return { code: current, map: null };
    },
  };
}
