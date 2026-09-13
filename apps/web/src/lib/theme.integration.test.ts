import { afterEach, describe, expect, it } from "vitest";
import {
  THEME_COOKIE_KEY,
  THEME_STORAGE_KEY,
  applyTheme,
  readThemePreference,
  resolveTheme,
  resolveThemeForSsr,
  themePreferenceFromCookieHeader,
} from "./theme";

describe("theme helpers", () => {
  afterEach(() => {
    localStorage.removeItem(THEME_STORAGE_KEY);
    document.documentElement.classList.remove("dark");
    document.cookie = `${THEME_COOKIE_KEY}=; path=/; max-age=0`;
  });

  it("defaults preference to system", () => {
    expect(readThemePreference()).toBe("system");
  });

  it("resolveTheme honors explicit light/dark", () => {
    expect(resolveTheme("light")).toBe("light");
    expect(resolveTheme("dark")).toBe("dark");
  });

  it("applyTheme persists preference, cookie, and toggles dark class", () => {
    applyTheme("dark");
    expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);
    expect(document.cookie).toMatch(/octanest-theme=dark/);

    applyTheme("light");
    expect(document.documentElement.classList.contains("dark")).toBe(false);
    expect(document.cookie).toMatch(/octanest-theme=light/);
  });

  it("parses theme cookie and resolves SSR highlight theme", () => {
    expect(themePreferenceFromCookieHeader("octanest-theme=dark")).toBe("dark");
    expect(
      themePreferenceFromCookieHeader("a=1; octanest-theme=system; b=2"),
    ).toBe("system");
    expect(themePreferenceFromCookieHeader(undefined)).toBeNull();

    expect(resolveThemeForSsr("dark")).toBe("dark");
    expect(resolveThemeForSsr("system", "dark")).toBe("dark");
    expect(resolveThemeForSsr(null, null)).toBe("light");
  });
});
