// Shared DOM, storage, clipboard, document-fetch, and ScrollSpy utilities.

/**
 * Attaches Declarative Shadow DOM (`<template shadowrootmode>`) manually when
 * content was inserted via `innerHTML` or `DOMParser` rather than initial parse.
 */
export function ensureShadowRoot(el: HTMLElement): void {
  if (el.shadowRoot) return;
  const tpl = el.querySelector<HTMLTemplateElement>(
    ":scope > template[shadowrootmode]",
  );
  if (!tpl) return;
  const mode =
    tpl.getAttribute("shadowrootmode") === "closed" ? "closed" : "open";
  const shadow = el.attachShadow({ mode });
  shadow.appendChild(tpl.content.cloneNode(true));
  tpl.remove();
}

export function rafThrottle<T extends (...args: never[]) => void>(
  fn: T,
): { (...args: Parameters<T>): void; cancel(): void } {
  let frame: number | null = null;
  const throttled = (...args: Parameters<T>) => {
    if (frame !== null) return;
    frame = requestAnimationFrame(() => {
      frame = null;
      fn(...args);
    });
  };
  throttled.cancel = () => {
    if (frame !== null) {
      cancelAnimationFrame(frame);
      frame = null;
    }
  };
  return throttled;
}

export const storage = {
  get(key: string): string | null {
    try {
      return localStorage.getItem(key);
    } catch {
      return null;
    }
  },
  set(key: string, value: string): void {
    try {
      localStorage.setItem(key, value);
    } catch {}
  },
  remove(key: string): void {
    try {
      localStorage.removeItem(key);
    } catch {}
  },
};

export async function copyText(text: string): Promise<boolean> {
  if (!text) return false;
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
    const textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.style.position = "fixed";
    textarea.style.opacity = "0";
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand("copy");
    textarea.remove();
    return true;
  } catch {
    return false;
  }
}

export function readHash(): string {
  const raw = (window.location.hash || "").slice(1);
  if (!raw) return "";
  try {
    return decodeURIComponent(raw);
  } catch {
    return raw;
  }
}

export function syncHeadingHash(id: string): void {
  if (!id || readHash() === id) return;
  try {
    history.replaceState(history.state, "", `#${encodeURIComponent(id)}`);
  } catch {}
}

const docCache = new Map<string, Document>();

export function invalidateDocumentCache(url?: string): void {
  if (!url) {
    docCache.clear();
    return;
  }
  docCache.delete(url.split("#")[0]);
}

export async function fetchDocument(
  url: string | null | undefined,
  bypassCache = false,
): Promise<Document | null> {
  if (!url) return null;
  const cleanUrl = url.split("#")[0];
  if (!bypassCache) {
    const cached = docCache.get(cleanUrl);
    if (cached) return cached;
  }
  try {
    const res = await fetch(
      cleanUrl,
      bypassCache ? { cache: "no-store" } : undefined,
    );
    if (!res.ok) return null;
    const html = await res.text();
    const doc = new DOMParser().parseFromString(html, "text/html");
    docCache.set(cleanUrl, doc);
    return doc;
  } catch {
    return null;
  }
}

export interface HeadingTarget {
  id: string;
  el: HTMLElement;
  link: HTMLElement | null;
}

export interface ScrollSpyOptions {
  container: HTMLElement;
  headingTargets: HeadingTarget[];
  tocLinks: HTMLElement[];
  headingLine?: number;
  onActiveChange?: (currentId: string) => void;
}

export interface ScrollSpyInstance {
  update(): void;
  markClickScrolling(duration?: number): void;
  destroy(): void;
}

export function createScrollSpy({
  container,
  headingTargets,
  tocLinks,
  headingLine = 80,
  onActiveChange,
}: ScrollSpyOptions): ScrollSpyInstance {
  let cursor = 0;
  let isClickScrolling = false;
  let clickTimer: ReturnType<typeof setTimeout> | null = null;

  const markClickScrolling = (duration = 800) => {
    isClickScrolling = true;
    if (clickTimer) clearTimeout(clickTimer);
    clickTimer = setTimeout(() => {
      isClickScrolling = false;
    }, duration);
  };

  const currentHeadingId = (): string | null => {
    const n = headingTargets.length;
    if (n === 0) return null;
    const line = container.getBoundingClientRect().top + headingLine;
    if (cursor >= n) cursor = n - 1;

    while (
      cursor > 0 &&
      headingTargets[cursor].el.getBoundingClientRect().top > line
    ) {
      cursor--;
    }
    while (
      cursor < n - 1 &&
      headingTargets[cursor + 1].el.getBoundingClientRect().top <= line
    ) {
      cursor++;
    }
    return headingTargets[cursor].id;
  };

  const update = () => {
    if (isClickScrolling) return;
    const currentId = currentHeadingId();
    if (!currentId) return;

    const active = headingTargets.find((item) => item.id === currentId);
    if (active?.link && !active.link.classList.contains("is-active")) {
      for (const l of tocLinks) l.classList.remove("is-active");
      active.link.classList.add("is-active");
      active.link.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
    onActiveChange?.(currentId);
  };

  const throttledUpdate = rafThrottle(update);
  container.addEventListener("scroll", throttledUpdate, { passive: true });

  return {
    update,
    markClickScrolling,
    destroy() {
      throttledUpdate.cancel();
      container.removeEventListener("scroll", throttledUpdate);
      if (clickTimer) clearTimeout(clickTimer);
    },
  };
}
