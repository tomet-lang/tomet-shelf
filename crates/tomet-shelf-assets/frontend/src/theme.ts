// Light / Dark theme controller.

import { storage } from "./dom";
import { t } from "./env";

export type ThemeMode = "light" | "dark";

export function getEffectiveTheme(): ThemeMode {
  const forced = document.documentElement.getAttribute("data-theme");
  if (forced === "dark" || forced === "light") return forced;
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function syncThemeLabels(): void {
  const isDark = getEffectiveTheme() === "dark";
  const label = isDark
    ? t("theme.to_light", "ライトテーマに切り替え")
    : t("theme.to_dark", "ダークテーマに切り替え");

  document
    .querySelectorAll<HTMLElement>(".theme-toggle-btn, #btn-header-theme")
    .forEach((btn) => {
      btn.setAttribute("title", label);
      btn.setAttribute("aria-label", label);
    });
}

export function toggleTheme(): void {
  const next: ThemeMode = getEffectiveTheme() === "dark" ? "light" : "dark";
  document.documentElement.setAttribute("data-theme", next);
  storage.set("shelf-theme", next);
  syncThemeLabels();
}

export function setupThemeToggle(): void {
  document
    .querySelectorAll<HTMLElement>(".theme-toggle-btn, #btn-header-theme")
    .forEach((btn) => {
      btn.onclick = (e) => {
        e.preventDefault();
        toggleTheme();
      };
    });
  syncThemeLabels();
}
