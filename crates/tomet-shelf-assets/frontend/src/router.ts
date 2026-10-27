// PJAX client-side router & shared page-region DOM swapper.

import { closeMobileNav, setupBookViewInteraction } from "./book-view";
import { closeCenterPeek } from "./center-peek";
import { setupDevTools } from "./dev-tools";
import { fetchDocument } from "./dom";
import { isInternalNoteUrl, normalizePath } from "./env";
import { setupInfoboxAndCopy } from "./infobox";
import { setupImageLightbox } from "./lightbox";
import { hidePreviewCard } from "./preview";
import { finishPageProgress, startPageProgress } from "./progress";
import { markVisited, updateRecentNotes } from "./recent-notes";

let isNavigating = false;
let currentPath = "";
let routerInitialized = false;

const ACTIVE_INDEX_CLASSES = [
  "is-here",
  "font-semibold",
  "text-accent-primary",
  "bg-subtle",
] as const;

export function reinitPageContent(): void {
  setupBookViewInteraction();
  setupImageLightbox();
  setupInfoboxAndCopy();
  setupDevTools();
  updateRecentNotes();
}

function saveCurrentScrollState(): void {
  const scrollContainer = document.getElementById("book-content-scroll");
  const scrollTop = scrollContainer
    ? scrollContainer.scrollTop
    : window.scrollY;
  try {
    history.replaceState(
      { ...(history.state || {}), scrollTop, path: window.location.href },
      document.title,
    );
  } catch {}
}

export interface SwapOptions {
  cleanUrl: string;
  replaceNavTree?: boolean;
}

/**
 * Replaces the note-specific DOM regions (`.header-breadcrumbs`,
 * `.article-container`, `#pane-context`, and sidebar active highlight) from a
 * parsed HTML document while keeping the outer shell intact.
 */
export function swapPageRegions(newDoc: Document, opts: SwapOptions): void {
  document.title = newDoc.title;

  const scriptEl = Array.from(newDoc.querySelectorAll("script")).find((s) =>
    (s.textContent || "").includes("window.tmtPageUrl"),
  );
  const match = scriptEl?.textContent?.match(
    /window\.tmtPageUrl\s*=\s*(".*?"|'.*?');/,
  );
  if (match) {
    try {
      window.tmtPageUrl = JSON.parse(match[1]) as string;
    } catch {
      window.tmtPageUrl = opts.cleanUrl;
    }
  } else {
    window.tmtPageUrl = opts.cleanUrl;
  }

  const currentCrumb = document.querySelector(".header-breadcrumbs");
  const newCrumb = newDoc.querySelector(".header-breadcrumbs");
  if (currentCrumb && newCrumb) {
    currentCrumb.innerHTML = newCrumb.innerHTML;
  }

  if (opts.replaceNavTree) {
    const currentIndex = document.querySelector("#pane-nav .pane-book-index");
    const newIndex = newDoc.querySelector("#pane-nav .pane-book-index");
    if (currentIndex && newIndex) {
      currentIndex.innerHTML = newIndex.innerHTML;
    }
  }

  const targetNorm = normalizePath(opts.cleanUrl);
  document
    .querySelectorAll<HTMLAnchorElement>("#pane-nav .book-index-link")
    .forEach((el) => {
      const isHere = normalizePath(el.getAttribute("href")) === targetNorm;
      for (const cls of ACTIVE_INDEX_CLASSES) {
        el.classList.toggle(cls, isHere);
      }
      if (isHere) el.setAttribute("aria-current", "page");
      else el.removeAttribute("aria-current");
    });

  const currentArticle = document.querySelector(".article-container");
  const newArticle = newDoc.querySelector(".article-container");
  if (currentArticle && newArticle) {
    currentArticle.innerHTML = newArticle.innerHTML;
  }

  const currentContext = document.getElementById("pane-context");
  const newContext = newDoc.getElementById("pane-context");
  if (currentContext && newContext) {
    currentContext.innerHTML = newContext.innerHTML;
  }
}

export async function navigateTo(
  url: string,
  pushState = true,
  savedScrollTop?: number | null,
): Promise<void> {
  if (isNavigating) return;

  const targetNorm = normalizePath(url);
  const rawHash = url.includes("#") ? url.split("#")[1] : "";
  const hash = rawHash ? decodeURIComponent(rawHash) : "";

  if (currentPath && currentPath === targetNorm) {
    const scrollContainer = document.getElementById("book-content-scroll");
    if (hash) {
      const targetEl = document.getElementById(hash);
      if (targetEl && scrollContainer) {
        const top =
          scrollContainer.scrollTop +
          targetEl.getBoundingClientRect().top -
          scrollContainer.getBoundingClientRect().top -
          16;
        scrollContainer.scrollTo({ top, behavior: "smooth" });
      }
    } else if (scrollContainer) {
      scrollContainer.scrollTo({ top: 0, behavior: "smooth" });
    }
    if (pushState) {
      history.pushState({ path: url, scrollTop: 0 }, "", url);
    }
    return;
  }

  if (pushState) saveCurrentScrollState();

  isNavigating = true;
  startPageProgress();

  closeCenterPeek();
  hidePreviewCard();
  closeMobileNav();
  const searchResults = document.getElementById("wiki-search-results");
  if (searchResults) searchResults.hidden = true;

  try {
    const cleanUrl = url.split("#")[0];
    const newDoc = await fetchDocument(cleanUrl);
    const hasCurrentShell = Boolean(document.querySelector(".app-layout"));
    const hasNewShell = Boolean(newDoc?.querySelector(".app-layout"));

    if (!newDoc || !hasCurrentShell || !hasNewShell) {
      finishPageProgress();
      if (!pushState) window.location.reload();
      else window.location.href = url;
      return;
    }

    const applySwap = () => {
      if (pushState) {
        history.pushState({ path: url, scrollTop: 0 }, newDoc.title, url);
      }
      currentPath = targetNorm;
      markVisited(cleanUrl);
      swapPageRegions(newDoc, { cleanUrl });

      const scrollContainer = document.getElementById("book-content-scroll");
      if (scrollContainer) {
        if (hash) {
          const targetEl = document.getElementById(hash);
          scrollContainer.scrollTop = targetEl
            ? scrollContainer.scrollTop +
              targetEl.getBoundingClientRect().top -
              scrollContainer.getBoundingClientRect().top -
              16
            : 0;
        } else if (savedScrollTop != null) {
          scrollContainer.scrollTop = savedScrollTop;
        } else {
          scrollContainer.scrollTop = 0;
        }
      }

      reinitPageContent();
      finishPageProgress();
    };

    if (document.startViewTransition) {
      document.startViewTransition(applySwap);
    } else {
      applySwap();
    }
  } catch (err) {
    console.warn("PJAX navigation failed, falling back:", err);
    finishPageProgress();
    if (!pushState) window.location.reload();
    else window.location.href = url;
  } finally {
    isNavigating = false;
  }
}

export function setupClientRouter(): void {
  if (routerInitialized) return;
  routerInitialized = true;

  currentPath = normalizePath(window.location.href);

  try {
    if (!history.state || history.state.scrollTop === undefined) {
      const scrollContainer = document.getElementById("book-content-scroll");
      const scrollTop = scrollContainer
        ? scrollContainer.scrollTop
        : window.scrollY;
      history.replaceState(
        { ...(history.state || {}), path: window.location.href, scrollTop },
        document.title,
      );
    }
  } catch {}

  document.addEventListener("click", (e) => {
    if (
      e.defaultPrevented ||
      e.metaKey ||
      e.ctrlKey ||
      e.shiftKey ||
      e.altKey ||
      e.button !== 0
    ) {
      return;
    }

    const a = (e.target as Element | null)?.closest("a");
    if (!a || a.getAttribute("target") === "_blank") return;
    if (
      a.classList.contains("tm-unresolved") ||
      a.classList.contains("tm-ref-unresolved")
    ) {
      return;
    }

    const href = a.getAttribute("href");
    if (!isInternalNoteUrl(href)) return;

    e.preventDefault();
    void navigateTo(href, true);
  });

  window.addEventListener("popstate", (e) => {
    const savedScroll =
      typeof e.state?.scrollTop === "number" ? e.state.scrollTop : null;
    void navigateTo(window.location.href, false, savedScroll);
  });
}
