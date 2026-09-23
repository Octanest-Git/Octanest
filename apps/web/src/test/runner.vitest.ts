/**
 * Vitest-facing test runner re-exports (default / Node / Vite resolution).
 * Bun test scripts use `runner.bun.ts` via package.json exports + `--conditions=octanest-bun-test`.
 */
export {
  afterAll,
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  test,
  vi,
} from "vitest";
