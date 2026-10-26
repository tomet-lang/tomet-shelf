(() => {
  const T = (window.TMT ??= {} as any);
  const t = (key: string, fallback = "") => T.t(key, fallback);

  // ==================== THEME CONTROLLER ====================
  function getSystemTheme() {
    return window.matchMedia &&
      window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }

  function getEffectiveTheme() {
    const forced = document.documentElement.getAttribute("data-theme");
    if (forced === "dark" || forced === "light") return forced;
    return getSystemTheme();
  }

  function syncThemeIcons() {
    const isDark = getEffectiveTheme() === "dark";
    const titleStr = isDark
      ? t("theme.to_light", "ライトテーマに切り替え")
      : t("theme.to_dark", "ダークテーマに切り替え");

    const themeBtns = document.querySelectorAll(
      ".theme-toggle-btn, #btn-rail-theme, #btn-header-theme",
    );
    themeBtns.forEach((btn) => {
      btn.setAttribute("title", titleStr);
      btn.setAttribute("aria-label", titleStr);
    });
  }

  function toggleTheme() {
    const current = getEffectiveTheme();
    const nextTheme = current === "dark" ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", nextTheme);
    T.storage.set("shelf-theme", nextTheme);
    syncThemeIcons();
  }

  function setupThemeToggle() {
    const themeBtns = document.querySelectorAll(
      ".theme-toggle-btn, #btn-rail-theme, #btn-header-theme",
    );
    themeBtns.forEach((btn) => {
      (btn as HTMLElement).onclick = (e) => {
        e.preventDefault();
        toggleTheme();
      };
    });
    syncThemeIcons();
  }

  Object.assign(T, {
    setupThemeToggle,
    getEffectiveTheme,
    toggleTheme,
    // Stub methods for backwards compatibility if needed
    setupShellTheme: () => {},
    getShellColor: () => "slate",
    setShellColor: () => {},
    getShellPattern: () => "none",
    setShellPattern: () => {},
    getShellContrast: () => "normal",
    setShellContrast: () => {},
    toggleShellContrast: () => {},
    getShellTheme: () => "slate",
    setShellTheme: () => {},
  });
})();
