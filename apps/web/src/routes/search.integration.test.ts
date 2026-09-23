import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "@octanest/web/test-runner";

const dir = dirname(fileURLToPath(import.meta.url));

describe("global search route", () => {
  it("defines SearchPage and Cookie-forward SSR loader", () => {
    const src = readFileSync(join(dir, "search.tsrx"), "utf8");
    expect(src).toMatch(/export function SearchPage/);
    expect(src).toMatch(/createFileRoute\("\/search"\)/);
    expect(src).toMatch(/createServerFn/);
  });
});
