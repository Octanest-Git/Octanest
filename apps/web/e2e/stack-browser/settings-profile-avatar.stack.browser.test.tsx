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

  it("covers home dashboard, general/theme, tokens nesting, profile, and anon redirect", async () => {
    const ok = await commands.expectSettingsProfileAvatarFlow();
    expect(ok).toBe(true);
  }, 300_000);
});
