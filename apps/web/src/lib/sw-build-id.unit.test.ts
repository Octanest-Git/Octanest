import { describe, expect, it } from "@octanest/web/test-runner";
import {
  resolveSwBuildId,
  sanitizeSwBuildId,
  stampSwSource,
  SW_BUILD_PLACEHOLDER,
} from "./sw-build-id";
import { buildSwRegisterScript } from "./sw-register";

describe("sw-build-id", () => {
  it("sanitizes to URL-safe short ids", () => {
    expect(sanitizeSwBuildId(" abc/def!0123456789abcdef ")).toBe("abcdef0123456789abcdef");
    expect(sanitizeSwBuildId("")).toBe("unknown");
  });

  it("prefers Railway / CI commit env over local fallback", () => {
    expect(
      resolveSwBuildId({
        RAILWAY_GIT_COMMIT_SHA: "deadbeefcafebabe",
      }),
    ).toBe("deadbeefcafebabe");
    expect(
      resolveSwBuildId({
        GITHUB_SHA: "1111222233334444",
        RAILWAY_GIT_COMMIT_SHA: "aaaabbbbccccdddd",
      }),
    ).toBe("aaaabbbbccccdddd");
  });

  it("stamps CACHE_NAME placeholder", () => {
    const src = `const CACHE_NAME = "octanest-shell-${SW_BUILD_PLACEHOLDER}";\n`;
    expect(stampSwSource(src, "abc123")).toContain('const CACHE_NAME = "octanest-shell-abc123";');
    expect(stampSwSource(src, "abc123")).not.toContain(SW_BUILD_PLACEHOLDER);
  });
});

describe("buildSwRegisterScript", () => {
  it("embeds build id and update/skipWaiting hooks", () => {
    const script = buildSwRegisterScript("deploy-1");
    expect(script).toContain('BUILD="deploy-1"');
    expect(script).toContain("/sw.js?v=");
    expect(script).toContain("reg.update()");
    expect(script).toContain("SKIP_WAITING");
    expect(script).toContain("controllerchange");
  });

  it("strips unsafe characters from build id", () => {
    const script = buildSwRegisterScript('x";alert(1)//');
    expect(script).toContain('BUILD="xalert1"');
    expect(script).not.toContain("alert(1)");
  });
});
