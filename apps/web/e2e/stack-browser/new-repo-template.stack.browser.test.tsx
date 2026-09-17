import { beforeAll, describe, expect, it } from "vitest";
import { commands } from "vitest/browser";
import { requireStack } from "../stack/env";

declare module "vitest/browser" {
  interface BrowserCommands {
    expectNewRepoTemplatePickerFlow: () => Promise<boolean>;
  }
}

describe("stack browser e2e: /new template picker", () => {
  beforeAll(() => {
    requireStack();
  });

  it("picks a stack without insertBefore / Octane pageerrors", async () => {
    const ok = await commands.expectNewRepoTemplatePickerFlow();
    expect(ok).toBe(true);
  }, 180_000);
});
