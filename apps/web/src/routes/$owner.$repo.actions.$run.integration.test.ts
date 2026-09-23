/**
 * Phase 19 — Actions run detail + job logs (ACT-03 / D-ACT-12 / D-ACT-13).
 */
import { describe, expect, it } from "@octanest/web/test-runner";

describe("/$owner/$repo/actions/$run", () => {
  it("shows run detail with job list and statuses", async () => {
    const src = await import("./$owner.$repo.actions.$run.tsrx?raw").then(
      (m) => m.default as string,
    );
    expect(src).toContain('data-testid="repo-actions-run"');
    expect(src).toContain("repo-actions-jobs");
    expect(src).toContain("actionsRunDetailQuery");
  });

  it("renders job log panel from Actions log store", async () => {
    const src = await import("./$owner.$repo.actions.$run.tsrx?raw").then(
      (m) => m.default as string,
    );
    expect(src).toContain("repo-actions-job-log");
    expect(src).toContain("actionsJobLogQuery");
  });

  it("inherits layout chrome — no duplicate RepoChrome remount (D-QH-01)", async () => {
    const src = await import("./$owner.$repo.actions.$run.tsrx?raw").then(
      (m) => m.default as string,
    );
    expect(src).not.toMatch(/RepoChrome/);
    expect(src).toContain('createFileRoute("/$owner/$repo/actions/$run")');
  });
});
