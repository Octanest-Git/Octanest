import { afterEach, describe, expect, it } from "vitest";
import {
  THEME_STORAGE_KEY,
  applyTheme,
  readThemePreference,
  resolveTheme,
} from "./theme";

describe("theme helpers", () => {
  afterEach(() => {
    localStorage.removeItem(THEME_STORAGE_KEY);
    document.documentElement.classList.remove("dark");
  });

  it("defaults preference to system", () => {
    expect(readThemePreference()).toBe("system");
  });

  it("resolveTheme honors explicit light/dark", () => {
    expect(resolveTheme("light")).toBe("light");
    expect(resolveTheme("dark")).toBe("dark");
  });

  it("applyTheme persists preference and toggles dark class", () => {
    applyTheme("dark");
    expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);

    applyTheme("light");
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });
});
