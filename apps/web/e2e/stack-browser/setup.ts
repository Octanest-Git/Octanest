import { beforeAll } from "vitest";

beforeAll(async () => {
  if (process.env.E2E_STACK !== "1") {
    return;
  }
  const web = (process.env.OCTANEST_E2E_WEB_ORIGIN || "http://127.0.0.1:13000").replace(
    /\/$/,
    "",
  );
  const res = await fetch(web).catch(() => null);
  if (!res?.ok) {
    throw new Error(
      `E2E_STACK=1 but web origin failed at ${web} — run make test-e2e-stack`,
    );
  }
});
