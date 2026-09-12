export type ThemePreference = "system" | "light" | "dark";

export const THEME_STORAGE_KEY = "octanest-theme";

export function readThemePreference(): ThemePreference {
  if (typeof localStorage === "undefined") return "system";
  const v = localStorage.getItem(THEME_STORAGE_KEY);
  if (v === "light" || v === "dark" || v === "system") return v;
  return "system";
}

export function resolveTheme(pref: ThemePreference): "light" | "dark" {
  if (pref === "light" || pref === "dark") return pref;
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

export function applyTheme(pref: ThemePreference) {
  const resolved = resolveTheme(pref);
  document.documentElement.classList.toggle("dark", resolved === "dark");
  localStorage.setItem(THEME_STORAGE_KEY, pref);
}

export const THEME_OPTIONS: readonly ThemePreference[] = ["system", "light", "dark"];

// Runs in <head> before first paint (D-12). Static literal — never interpolate
// request data or storage values into this string (T-03-04).
export const THEME_BOOT_SCRIPT =
  '(function(){try{var v=localStorage.getItem("octanest-theme");' +
  'var p=(v==="light"||v==="dark"||v==="system")?v:"system";' +
  'var d=p==="dark"||(p==="system"&&window.matchMedia("(prefers-color-scheme: dark)").matches);' +
  'document.documentElement.classList.toggle("dark",d);}catch(e){}})();';

/**
 * Warm webfonts into `document.fonts` so View Transitions / soft navigations
 * do not paint a long stretch of metric fallbacks after CSS re-applies.
 * Static literal — no interpolation (same threat model as theme boot).
 */
export const FONT_WARM_SCRIPT =
  '(function(){try{if(!document.fonts||!document.fonts.load)return;' +
  'document.fonts.load(\'600 24px "Sora Variable"\');' +
  'document.fonts.load(\'400 16px "Source Sans 3 Variable"\');' +
  '}catch(e){}})();';
