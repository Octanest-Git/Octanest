import { beforeAll, describe, expect, it } from "bun:test";

import { requireStack } from "../lib/env.ts";
import { expectSettingsProfileAvatarFlow } from "../lib/flows.ts";

describe("bun.webview PoC: settings profile avatar", () => {
  beforeAll(() => {
    requireStack();
  });

  it("renders settings pages and profile avatar controls", async () => {
    expect(await expectSettingsProfileAvatarFlow()).toBe(true);
  }, 120_000);
});
