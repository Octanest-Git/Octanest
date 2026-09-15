import { beforeAll, describe, expect, it } from "vitest";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectSettingsProfileAvatarFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: settings SSR + profile avatar crop/remove", () => {
  beforeAll(() => {
    requireStack();
  });

  it("loads settings pages SSR-first and covers avatar reject/crop/save/remove", async () => {
    const ok = await commands.expectSettingsProfileAvatarFlow();
    expect(ok).toBe(true);
  }, 240_000);
});
