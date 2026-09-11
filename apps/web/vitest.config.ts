import path from "node:path";
import { fileURLToPath } from "node:url";
import { octane } from "@octanejs/vite-plugin";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";
import {
  ensureAuthSettings,
  expectWorkosCta,
  restoreLocalAuthCommand,
  signupThroughUi,
} from "./e2e/stack-browser/commands";

const rootDir = path.dirname(fileURLToPath(import.meta.url));
const stackEnabled = process.env.E2E_STACK === "1";

/** Static `process.env.KEY` replacements so browser tests see harness env. */
const stackEnvDefine = {
  "process.env.E2E_STACK": JSON.stringify(process.env.E2E_STACK ?? ""),
  "process.env.OCTANEST_E2E_API_ORIGIN": JSON.stringify(
    process.env.OCTANEST_E2E_API_ORIGIN ?? "http://127.0.0.1:18080",
  ),
  "process.env.OCTANEST_E2E_WEB_ORIGIN": JSON.stringify(
    process.env.OCTANEST_E2E_WEB_ORIGIN ?? "http://127.0.0.1:13000",
  ),
  "process.env.OCTANEST_E2E_MAILPIT_ORIGIN": JSON.stringify(
    process.env.OCTANEST_E2E_MAILPIT_ORIGIN ?? "http://127.0.0.1:8025",
  ),
  "process.env.OCTANEST_E2E_STUBS_ORIGIN": JSON.stringify(
    process.env.OCTANEST_E2E_STUBS_ORIGIN ?? "http://127.0.0.1:9092",
  ),
  "process.env.OCTANEST_E2E_ADMIN_EMAIL": JSON.stringify(
    process.env.OCTANEST_E2E_ADMIN_EMAIL ?? "admin@octanest.local",
  ),
  "process.env.OCTANEST_E2E_ADMIN_PASSWORD": JSON.stringify(
    process.env.OCTANEST_E2E_ADMIN_PASSWORD ?? "password1",
  ),
  "process.env.OCTANEST_E2E_OIDC_ISSUER": JSON.stringify(
    process.env.OCTANEST_E2E_OIDC_ISSUER ?? "http://127.0.0.1:9090/default",
  ),
  "process.env.OCTANEST_E2E_DB_PATH": JSON.stringify(
    process.env.OCTANEST_E2E_DB_PATH ?? "",
  ),
};

/**
 * Vitest projects:
 * - unit / integration / e2e-component: always on in `bun run test`
 * - e2e-stack (+ browser): only when E2E_STACK=1 (`make test-e2e-stack`)
 */
export default defineConfig({
  plugins: [octane()],
  resolve: {
    alias: {
      "@": path.resolve(rootDir, "./src"),
    },
  },
  test: {
    globals: false,
    projects: [
      {
        extends: true,
        test: {
          name: "unit",
          environment: "node",
          include: ["src/**/*.unit.test.ts", "src/**/*.gate.test.ts"],
        },
      },
      {
        extends: true,
        test: {
          name: "integration",
          environment: "happy-dom",
          include: ["src/**/*.integration.test.{ts,tsx}"],
          setupFiles: ["./src/test/setup-integration.ts"],
        },
      },
      {
        extends: true,
        test: {
          name: "e2e-component",
          include: ["e2e/component/**/*.e2e.test.{ts,tsx}"],
          setupFiles: ["./e2e/component/setup.ts"],
          browser: {
            enabled: true,
            provider: playwright(),
            headless: true,
            instances: [{ browser: "chromium" }],
          },
        },
      },
      ...(stackEnabled
        ? [
            {
              extends: true as const,
              define: stackEnvDefine,
              test: {
                name: "e2e-stack",
                environment: "node" as const,
                include: ["e2e/stack/**/*.stack.test.ts"],
                fileParallelism: false,
                setupFiles: ["./e2e/stack/setup.ts"],
                testTimeout: 60_000,
              },
            },
            {
              extends: true as const,
              define: stackEnvDefine,
              test: {
                name: "e2e-stack-browser",
                include: ["e2e/stack-browser/**/*.stack.browser.test.{ts,tsx}"],
                setupFiles: ["./e2e/stack-browser/setup.ts"],
                fileParallelism: false,
                testTimeout: 60_000,
                browser: {
                  enabled: true,
                  provider: playwright(),
                  headless: true,
                  instances: [{ browser: "chromium" as const }],
                  commands: {
                    ensureAuthSettings,
                    restoreLocalAuthCommand,
                    signupThroughUi,
                    expectWorkosCta,
                  },
                },
              },
            },
          ]
        : []),
    ],
  },
});
