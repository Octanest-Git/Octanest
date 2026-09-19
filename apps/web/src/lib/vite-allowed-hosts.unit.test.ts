import { describe, expect, it } from "vitest";
import { parseViteAllowedHosts } from "../../vite-plugins/vite-allowed-hosts.ts";

describe("parseViteAllowedHosts", () => {
  it("returns empty when unset", () => {
    expect(parseViteAllowedHosts(undefined, undefined)).toEqual([]);
    expect(parseViteAllowedHosts("", "")).toEqual([]);
  });

  it("splits and trims comma-separated hosts", () => {
    expect(parseViteAllowedHosts(" .up.railway.app , Octanest.jereko.dev ", undefined)).toEqual([
      ".up.railway.app",
      "octanest.jereko.dev",
    ]);
  });

  it("adds hostname from OCTANEST_PUBLIC_ORIGIN", () => {
    expect(parseViteAllowedHosts(".up.railway.app", "https://octanest.jereko.dev/")).toEqual([
      ".up.railway.app",
      "octanest.jereko.dev",
    ]);
  });

  it("ignores malformed public origin", () => {
    expect(parseViteAllowedHosts("app.example", "not-a-url")).toEqual(["app.example"]);
  });
});
