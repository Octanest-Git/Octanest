/**
 * bun:test-facing runner re-exports (package.json exports condition `"bun"`).
 * Includes `vi` so Vitest-style mocks work under Bun without React assumptions —
 * UI under test is Octane (`.tsrx`).
 */
import {
  afterAll,
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  mock,
  spyOn,
  test,
  vi as bunVi,
} from "bun:test";

/** Vitest `vi.mocked()` compatibility — identity cast for typed mock access. */
function mocked<T>(item: T): T {
  return item;
}

export const vi = Object.assign(bunVi, { mocked });

export {
  afterAll,
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  mock,
  spyOn,
  test,
};
