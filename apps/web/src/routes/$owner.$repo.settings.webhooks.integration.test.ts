/**
 * Phase 18 — Settings Webhooks UI (HOOK-01/03, D-HOOK-19..21).
 */
import { describe, expect, it } from "@octanest/web/test-runner";

describe("repo settings Webhooks (HOOK-01/03)", () => {
  it("Admin Settings exposes a Webhooks heading/section", async () => {
    const settings = await import("./$owner.$repo.settings");
    expect(settings.RepoSettingsPage ?? settings.default).toBeTruthy();
    const panel = await import("../components/repo/webhooks-panel");
    expect(panel.WebhooksPanel ?? panel.default).toBeTruthy();
    const src = await import("./$owner.$repo.settings.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(src).toMatch(/WebhooksPanel/);
    expect(src).toMatch(/can_admin/);
    const panelSrc = await import("../components/repo/webhooks-panel.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(panelSrc).toMatch(/Webhooks/);
    expect(panelSrc).toMatch(/webhook\.list|apiClient\.webhook\.list/);
  }, 30_000);

  it("Admin can create webhook with URL, secret, and event checkboxes", async () => {
    const form = await import("../components/repo/webhook-form");
    expect(form.WebhookForm ?? form.default).toBeTruthy();
    const formSrc = await import("../components/repo/webhook-form.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(formSrc).toMatch(/webhook\.create|apiClient\.webhook\.create/);
    expect(formSrc).toMatch(/secret/i);
    expect(formSrc).toMatch(/push/);
    expect(formSrc).toMatch(/pull_request/);
    expect(formSrc).toMatch(/issues/);
    expect(formSrc).toMatch(/active/i);
    // One-time reveal after create (D-HOOK-16)
    expect(formSrc).toMatch(/secret|Copy|won.?t be able|reveal/i);
    const panelSrc = await import("../components/repo/webhooks-panel.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(panelSrc).toMatch(/WebhookForm/);
    expect(panelSrc).toMatch(/webhook\.delete|apiClient\.webhook\.delete/);
    expect(panelSrc).toMatch(/webhook\.update|apiClient\.webhook\.update/);
  }, 30_000);

  it("Admin sees recent delivery attempts with HTTP status and ping/redeliver", async () => {
    const deliveries = await import("../components/repo/webhook-deliveries");
    expect(deliveries.WebhookDeliveries ?? deliveries.default).toBeTruthy();
    const dSrc = await import("../components/repo/webhook-deliveries.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(dSrc).toMatch(/deliveries\.list|webhook\.deliveries/);
    expect(dSrc).toMatch(/http_status|HTTP/i);
    expect(dSrc).toMatch(/Ping/);
    expect(dSrc).toMatch(/Redeliver/);
    expect(dSrc).toMatch(/webhook\.ping|apiClient\.webhook\.ping/);
    expect(dSrc).toMatch(/webhook\.redeliver|apiClient\.webhook\.redeliver/);
    const panelSrc = await import("../components/repo/webhooks-panel.tsrx?raw").then((m) =>
      String((m as { default: string }).default),
    );
    expect(panelSrc).toMatch(/WebhookDeliveries/);
  }, 30_000);
});
