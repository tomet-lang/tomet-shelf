// Hover Page Preview card (Obsidian / Wikipedia style).

import { openCenterPeek } from "./center-peek";
import { fetchDocument } from "./dom";
import { isInternalNoteUrl } from "./env";

interface PreviewSummary {
  title: string;
  section: string;
  excerpt: string;
  image: string;
}

const previewCache = new Map<string, PreviewSummary>();
let hoverTimer: ReturnType<typeof setTimeout> | null = null;
let activeLink: HTMLAnchorElement | null = null;
let initialized = false;

export function hidePreviewCard(): void {
  const card = document.getElementById("wiki-page-preview");
  if (card) card.hidden = true;
  activeLink = null;
}

export function invalidatePreviewCache(): void {
  previewCache.clear();
}

export function initPagePreview(): void {
  if (initialized) return;
  initialized = true;

  window.addEventListener("scroll", hidePreviewCard, {
    capture: true,
    passive: true,
  });

  document.addEventListener("click", (e) => {
    const card = document.getElementById("wiki-page-preview");
    if (card && !card.contains(e.target as Node | null)) {
      hidePreviewCard();
    }
  });

  const cardEl = document.getElementById("wiki-page-preview");
  if (cardEl) {
    cardEl.addEventListener("click", () => {
      const href = cardEl.dataset.href;
      if (href) {
        hidePreviewCard();
        void openCenterPeek(href);
      }
    });
    cardEl.addEventListener("mouseleave", () => {
      if (hoverTimer) clearTimeout(hoverTimer);
      hoverTimer = setTimeout(() => {
        if (!cardEl.matches(":hover")) hidePreviewCard();
      }, 150);
    });
  }

  document.addEventListener("mouseover", (e) => {
    const card = document.getElementById("wiki-page-preview");
    if (!card) return;

    const link = (e.target as Element | null)?.closest("a");
    if (!link) return;
    if (
      link.classList.contains("tm-unresolved") ||
      link.classList.contains("tm-ref-unresolved")
    ) {
      return;
    }

    const inPreviewScope = link.closest(
      "article, #wiki-search-results, #pane-recent-section, .backlink-items",
    );
    if (!inPreviewScope) return;

    const href = link.getAttribute("href");
    if (!isInternalNoteUrl(href)) return;

    activeLink = link;
    if (hoverTimer) clearTimeout(hoverTimer);

    hoverTimer = setTimeout(async () => {
      if (activeLink !== link) return;

      let summary = previewCache.get(href);
      if (!summary) {
        const doc = await fetchDocument(href);
        if (!doc) return;

        const title =
          doc.querySelector(".content-title")?.textContent?.trim() ||
          doc.querySelector("h1")?.textContent?.trim() ||
          "";
        const section =
          doc.querySelector(".crumb-section")?.textContent?.trim() || "";
        const excerpt =
          doc
            .querySelector("article p, article li, article blockquote")
            ?.textContent?.trim() || "";
        const imgEl = doc.querySelector(
          ".hero-banner-img, .infobox-main-img, .hero-avatar img, article img:not(.tm-link-icon):not(.tm-doc-icon-img)",
        );
        const image = imgEl?.getAttribute("src") || "";

        summary = { title, section, excerpt, image };
        previewCache.set(href, summary);
      }

      if (activeLink !== link) return;
      if (!summary.excerpt && !summary.title) return;

      const titleEl = card.querySelector(".preview-title");
      const sectionEl = card.querySelector(".preview-section");
      const bodyEl = card.querySelector(".preview-body");
      const thumbEl = card.querySelector<HTMLImageElement>(".preview-thumb");

      if (titleEl) titleEl.textContent = summary.title;
      if (sectionEl) {
        sectionEl.textContent = summary.section ? `(${summary.section})` : "";
      }
      if (bodyEl) bodyEl.textContent = summary.excerpt;

      if (thumbEl) {
        if (summary.image) {
          thumbEl.src = summary.image;
          thumbEl.hidden = false;
        } else {
          thumbEl.src = "";
          thumbEl.hidden = true;
        }
      }

      const rect = link.getBoundingClientRect();
      const cardWidth = 320;
      let left = rect.left;
      let top = rect.bottom + 8;

      if (
        link.closest("#pane-recent-section") &&
        rect.right + cardWidth + 24 <= window.innerWidth
      ) {
        left = rect.right + 12;
        top = Math.max(16, Math.min(rect.top, window.innerHeight - 180));
      } else {
        if (left + cardWidth > window.innerWidth - 16) {
          left = window.innerWidth - cardWidth - 16;
        }
        if (left < 16) left = 16;
        if (top + 150 > window.innerHeight && rect.top > 160) {
          top = rect.top - 150;
        }
      }

      card.dataset.href = href;
      card.style.left = `${Math.round(left)}px`;
      card.style.top = `${Math.round(top)}px`;
      card.hidden = false;
    }, 280);
  });

  document.addEventListener("mouseout", (e) => {
    const card = document.getElementById("wiki-page-preview");
    if (!card) return;
    const link = (e.target as Element | null)?.closest("a");
    if (link && link === activeLink) {
      activeLink = null;
      if (hoverTimer) clearTimeout(hoverTimer);
      hoverTimer = setTimeout(() => {
        if (!card.matches(":hover")) hidePreviewCard();
      }, 150);
    }
  });
}
