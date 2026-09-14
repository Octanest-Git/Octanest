import { describe, expect, it } from "vitest";
import {
  httpsCloneUrl,
  resolvePublicOriginFromEnv,
  resolveSshHost,
  resolveSshPort,
  sshCloneUrl,
  sshNeedsPortHint,
} from "./public-origin";

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

  it("builds scp-style SSH URLs for port 22 and 2222 (never ssh://)", () => {
    expect(sshCloneUrl("git.example", 22, "ada", "hello")).toBe(
      "git@git.example:ada/hello.git",
    );
    expect(sshCloneUrl("localhost/", 2222, "ada", "hello")).toBe(
      "git@localhost:ada/hello.git",
    );
    expect(sshCloneUrl("127.0.0.1", 2222, "ada", "hello")).not.toMatch(
      /^ssh:\/\//,
    );
  });

  it("sshNeedsPortHint only when port !== 22", () => {
    expect(sshNeedsPortHint(22)).toBe(false);
    expect(sshNeedsPortHint(2222)).toBe(true);
  });

  it("resolveSshHost prefers OCTANEST_SSH_HOST then origin hostname", () => {
    expect(resolveSshHost("http://127.0.0.1:3000", undefined)).toBe(
      "127.0.0.1",
    );
    expect(resolveSshHost("http://127.0.0.1:3000", "git.example")).toBe(
      "git.example",
    );
  });

  it("resolveSshPort defaults to 2222", () => {
    expect(resolveSshPort(undefined)).toBe(2222);
    expect(resolveSshPort("22")).toBe(22);
  });
});
