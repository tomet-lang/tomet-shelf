// Page configuration, i18n strings, and URL path helpers.

export function t(key: string, fallback = ""): string {
  return window.tmtStrings?.[key] ?? fallback;
}

export function basePath(): string {
  return (window.tmtBasePath || "").replace(/\/+$/, "");
}

export function withBasePath(path: string): string {
  const base = basePath();
  const suffix = path.startsWith("/") ? path : `/${path}`;
  return `${base}${suffix}`;
}

export function urlPrefix(): string {
  return window.tmtFullUrlPrefix || window.tmtUrlPrefix || "/wiki";
}

/**
 * Normalizes a URL or pathname into a canonical pathname with no `#hash`,
 * no `/index.html`, and no trailing slash (except `/` itself).
 */
export function normalizePath(input: string | null | undefined): string {
  if (!input) return "";
  try {
    const u = new URL(input, window.location.origin);
    return u.pathname.replace(/\/index\.html$/i, "").replace(/(.)\/+$/, "$1");
  } catch {
    return input
      .split("#")[0]
      .replace(/\/index\.html$/i, "")
      .replace(/(.)\/+$/, "$1");
  }
}

export function isHomeOrRootPath(pathname: string): boolean {
  const p = normalizePath(pathname);
  if (!p || p === "/") return true;
  const base = normalizePath(basePath());
  const home = normalizePath(window.tmtHomeUrl || "/wiki");
  const prefix = normalizePath(urlPrefix());
  return p === base || p === home || p === prefix;
}

/**
 * Returns true when `href` targets an internal note page within this vault
 * (not an external link, hash-only anchor, or catalog index root).
 */
export function isInternalNoteUrl(href: string | null | undefined): href is string {
  if (!href) return false;
  if (
    href.startsWith("#") ||
    href.startsWith("http://") ||
    href.startsWith("https://") ||
    href.startsWith("//") ||
    href.startsWith("mailto:") ||
    href.startsWith("javascript:") ||
    href.startsWith("vscode://") ||
    href.startsWith("cursor://")
  ) {
    return false;
  }

  let parsed: URL;
  try {
    parsed = new URL(href, window.location.href);
  } catch {
    return false;
  }
  if (parsed.origin !== window.location.origin) return false;

  const norm = normalizePath(parsed.pathname);
  if (isHomeOrRootPath(norm)) return false;

  const prefix = normalizePath(urlPrefix());
  return prefix ? norm.startsWith(`${prefix}/`) : norm.startsWith("/");
}
