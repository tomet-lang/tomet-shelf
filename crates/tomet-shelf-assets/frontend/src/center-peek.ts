// Center Peek modal (Notion / Obsidian style) + Shift+Click trigger.

import { fetchDocument } from "./dom";
import { isInternalNoteUrl, t } from "./env";
import { openLightbox } from "./lightbox";
import { hidePreviewCard } from "./preview";
import { navigateTo } from "./router";

let activePeekUrl: string | null = null;
let initialized = false;

export function closeCenterPeek(): void {
  const overlay = document.getElementById("wiki-center-peek");
  if (!overlay || overlay.hidden) return;
  overlay.hidden = true;
  activePeekUrl = null;
}

function setBodyMessage(
  bodyEl: HTMLElement,
  message: string,
  isError = false,
): void {
  const div = document.createElement("div");
  div.className = "peek-loading";
  if (isError) div.style.color = "var(--link-unresolved)";
  div.textContent = message;
  bodyEl.replaceChildren(div);
}

export async function openCenterPeek(url: string): Promise<void> {
  const overlay = document.getElementById("wiki-center-peek");
  const crumbsEl = document.getElementById("peek-crumbs");
  const bodyEl = document.getElementById("peek-body");
  if (!overlay || !crumbsEl || !bodyEl) return;

  hidePreviewCard();
  activePeekUrl = url;
  overlay.hidden = false;

  const loadingSpan = document.createElement("span");
  loadingSpan.className = "crumb-title";
  loadingSpan.textContent = t("search.loading", "読み込み中...");
  crumbsEl.replaceChildren(loadingSpan);
  setBodyMessage(bodyEl, t("search.loading", "読み込み中..."));

  const doc = await fetchDocument(url);
  if (activePeekUrl !== url) return;
  if (!doc) {
    setBodyMessage(
      bodyEl,
      t("peek.error", "ページの読み込みに失敗しました"),
      true,
    );
    return;
  }

  const title =
    doc.querySelector(".content-title")?.textContent?.trim() ||
    doc.querySelector("h1")?.textContent?.trim() ||
    "";
  const section =
    doc.querySelector(".crumb-section")?.textContent?.trim() || "";
  const iconEl = doc.querySelector(
    ".hero-avatar .avatar-icon, .title-icon, .infobox-title .infobox-icon",
  );

  const crumbNodes: Node[] = [];
  if (iconEl) {
    const span = document.createElement("span");
    span.className = "crumb-icon";
    if (iconEl.querySelector("svg, img")) {
      span.innerHTML = iconEl.innerHTML.trim();
    } else {
      span.textContent = iconEl.textContent?.trim() || "";
    }
    crumbNodes.push(span);
  }
  if (section) {
    const sectionSpan = document.createElement("span");
    sectionSpan.textContent = section;
    const sep = document.createElement("span");
    sep.style.opacity = "0.4";
    sep.textContent = "/";
    crumbNodes.push(sectionSpan, sep);
  }
  const titleSpan = document.createElement("span");
  titleSpan.className = "crumb-title";
  titleSpan.textContent = title || url;
  crumbNodes.push(titleSpan);
  crumbsEl.replaceChildren(...crumbNodes);

  const articleContainer = doc.querySelector(".article-container");
  if (!articleContainer) {
    setBodyMessage(bodyEl, t("peek.not_found", "本文が見つかりませんでした"));
    return;
  }

  const cloned = articleContainer.cloneNode(true) as HTMLElement;
  cloned
    .querySelectorAll(".hero-actions, .header-actions")
    .forEach((el) => el.remove());

  bodyEl.replaceChildren(cloned);
  bodyEl.scrollTop = 0;

  cloned.querySelectorAll<HTMLAnchorElement>("a").forEach((link) => {
    const href = link.getAttribute("href");
    if (!href) return;

    if (href.startsWith("#")) {
      link.addEventListener("click", (e) => {
        e.preventDefault();
        const targetId = decodeURIComponent(href.slice(1));
        const targetEl = bodyEl.querySelector(
          `[id="${CSS.escape(targetId)}"]`,
        );
        targetEl?.scrollIntoView({ behavior: "smooth", block: "start" });
      });
      return;
    }

    if (
      href.startsWith("http://") ||
      href.startsWith("https://") ||
      href.startsWith("mailto:")
    ) {
      link.setAttribute("target", "_blank");
      link.setAttribute("rel", "noopener noreferrer");
      return;
    }

    if (isInternalNoteUrl(href)) {
      link.addEventListener("click", (e) => {
        if (e.metaKey || e.ctrlKey || e.button !== 0) return;
        e.preventDefault();
        if (e.shiftKey) {
          void openCenterPeek(href);
        } else {
          closeCenterPeek();
          void navigateTo(href, true);
        }
      });
    }
  });

  cloned
    .querySelectorAll<HTMLImageElement>(
      "article img:not(.tm-link-icon):not(.tm-doc-icon-img), .hero-banner-img, .infobox-main-img",
    )
    .forEach((img) => {
      img.style.cursor = "zoom-in";
      img.addEventListener("click", () => {
        const src =
          img.getAttribute("data-original") || img.getAttribute("src");
        if (src) openLightbox(src, img.getAttribute("alt") || "");
      });
    });
}

export function initCenterPeek(): void {
  if (initialized) return;
  initialized = true;

  const overlay = document.getElementById("wiki-center-peek");
  if (!overlay) return;

  document
    .getElementById("peek-btn-close")
    ?.addEventListener("click", closeCenterPeek);

  document
    .getElementById("peek-btn-fullscreen")
    ?.addEventListener("click", () => {
      if (!activePeekUrl) return;
      const targetUrl = activePeekUrl;
      closeCenterPeek();
      void navigateTo(targetUrl, true);
    });

  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) closeCenterPeek();
  });

  // Shift + Click on any resolved internal note link opens Center Peek directly.
  document.addEventListener("click", (e) => {
    if (!e.shiftKey || e.metaKey || e.ctrlKey || e.altKey || e.button !== 0) {
      return;
    }
    const link = (e.target as Element | null)?.closest("a");
    if (!link) return;
    if (
      link.classList.contains("tm-unresolved") ||
      link.classList.contains("tm-ref-unresolved") ||
      link.closest("#wiki-center-peek")
    ) {
      return;
    }
    const href = link.getAttribute("href");
    if (!isInternalNoteUrl(href)) return;

    e.preventDefault();
    void openCenterPeek(href);
  });

  window.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !overlay.hidden) {
      const lightbox = document.getElementById("wiki-image-lightbox");
      if (lightbox && !lightbox.hidden) return;
      closeCenterPeek();
    }
  });
}
