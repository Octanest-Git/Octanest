import { describe, expect, it } from "vitest";

/**
 * GIT-11 Wave 0 stubs: /settings/tokens list / create / revoke UI
 * (D-14, D-15, D-17, D-24 / T-08-01 / T-08-03).
 *
 * RED until tokens.tsrx + create/reveal land (08-09–08-11).
 * Do not implement production routes here.
 *
 * Uses a variable dynamic import + @vite-ignore so Vitest can collect the
 * suite while ./tokens is still absent (static "./tokens" fails transform).
 */

/** Load tokens page; path is runtime-only so Vite does not resolve at transform. */
async function loadTokensModule(): Promise<Record<string, unknown>> {
  const rel = "./tokens";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /settings/tokens route missing — implement in 08-09 (GIT-11 / D-14). Expected list title Personal access tokens. ${(err as Error).message}`,
    );
  }
}

async function loadTokensNewModule(): Promise<Record<string, unknown>> {
  const rel = "./tokens.new";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /settings/tokens/new missing — implement in 08-10 (GIT-11 / D-15). Expected Make sure to copy your personal access token now. ${(err as Error).message}`,
    );
  }
}

describe("/settings/tokens Wave 0 (GIT-11 / D-14 list)", () => {
  it("list title Personal access tokens + empty hero No personal access tokens + Generate new token", async () => {
    const mod = await loadTokensModule();
    // Greened in 08-09: render TokensPage and assert:
    // - heading Personal access tokens
    // - empty hero No personal access tokens
    // - Generate new token dropdown → Classic token / Fine-grained token
    // - T-08-01: no plaintext ona_pat_ / ona_fg_ secrets on list
    expect(
      mod.TokensPage ?? mod.SettingsTokensPage ?? mod.default,
      "Wave 0: tokens module must export TokensPage for Personal access tokens list",
    ).toBeTruthy();
  });

  it("unverified: list visible with Generate disabled + Verify your email to create a token.", async () => {
    const mod = await loadTokensModule();
    // Greened in 08-09 (T-08-03 / D-24): email_verified=false → Generate new token
    // disabled + hint Verify your email to create a token. (list still visible)
    expect(
      mod.TokensPage ?? mod.SettingsTokensPage ?? mod.default,
      "Wave 0: unverified Generate gate — Verify your email to create a token.",
    ).toBeTruthy();
  });
});

describe("/settings/tokens Wave 0 (GIT-11 / D-17 revoke)", () => {
  it("revoke AlertDialog copy Revoke token? / Keep token", async () => {
    const mod = await loadTokensModule();
    // Greened in 08-09: Revoke token opens AlertDialog Revoke token? with Keep token dismiss
    expect(
      mod.TokensPage ?? mod.SettingsTokensPage ?? mod.default,
      "Wave 0: revoke dialog must use Revoke token? / Keep token (not Cancel)",
    ).toBeTruthy();
  });
});

describe("/settings/tokens Wave 0 (GIT-11 / D-15 one-time reveal)", () => {
  it("one-time reveal Make sure to copy your personal access token now", async () => {
    const mod = await loadTokensNewModule();
    // Greened in 08-10: after createClassic, reveal panel with
    // Make sure to copy your personal access token now (never on list — T-08-01)
    expect(
      mod.TokensNewPage ?? mod.ClassicCreatePage ?? mod.default,
      "Wave 0: create reveal must show Make sure to copy your personal access token now",
    ).toBeTruthy();
  });
});
