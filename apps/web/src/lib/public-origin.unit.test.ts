import { describe, expect, it } from "vitest";
import { httpsCloneUrl, resolvePublicOriginFromEnv } from "./public-origin";

describe("public-origin", () => {
  it("builds absolute HTTPS clone URLs", () => {
    expect(httpsCloneUrl("http://localhost", "ada", "hello")).toBe(
      "http://localhost/ada/hello.git",
    );
    expect(httpsCloneUrl("http://localhost/", "ada", "hello")).toBe(
      "http://localhost/ada/hello.git",
    );
  });

  it("reads OCTANEST_PUBLIC_ORIGIN when set", () => {
    const prev = process.env.OCTANEST_PUBLIC_ORIGIN;
    process.env.OCTANEST_PUBLIC_ORIGIN = "https://git.example/";
    expect(resolvePublicOriginFromEnv()).toBe("https://git.example");
    if (prev === undefined) delete process.env.OCTANEST_PUBLIC_ORIGIN;
    else process.env.OCTANEST_PUBLIC_ORIGIN = prev;
  });
});
