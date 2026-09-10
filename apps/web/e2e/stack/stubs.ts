import { stubsOrigin } from "./env";

export type StubLogEntry = {
  at: string;
  method: string;
  path: string;
  body?: unknown;
};

export async function stubsReset(): Promise<void> {
  const res = await fetch(`${stubsOrigin()}/__test/reset`, { method: "POST" });
  if (!res.ok) throw new Error(`stubs reset failed: ${res.status}`);
}

export async function stubsLog(): Promise<StubLogEntry[]> {
  const res = await fetch(`${stubsOrigin()}/__test/log`);
  if (!res.ok) throw new Error(`stubs log failed: ${res.status}`);
  const body = (await res.json()) as { entries?: StubLogEntry[] };
  return body.entries ?? [];
}

export async function waitForStub(
  predicate: (e: StubLogEntry) => boolean,
  timeoutMs = 15_000,
): Promise<StubLogEntry> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const entries = await stubsLog();
    const hit = entries.find(predicate);
    if (hit) return hit;
    await new Promise((r) => setTimeout(r, 200));
  }
  throw new Error("timed out waiting for stub request");
}
