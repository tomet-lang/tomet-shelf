// 3-pane book view interactions: mobile navigation drawer & TOC ScrollSpy.

import {
  createScrollSpy,
  type HeadingTarget,
  readHash,
  type ScrollSpyInstance,
  syncHeadingHash,
} from "./dom";

let activeSpy: ScrollSpyInstance | null = null;
let drawerEscapeBound = false;

export function closeMobileNav(): void {
  const paneNav = document.getElementById("pane-nav");
  const backdrop = document.getElementById("pane-nav-backdrop");
  const toggleBtn = document.getElementById("btn-toggle-nav");
  paneNav?.classList.remove("is-open");
  backdrop?.classList.remove("is-open");
  toggleBtn?.setAttribute("aria-expanded", "false");
}

function setMobileNavOpen(open: boolean): void {
  const paneNav = document.getElementById("pane-nav");
  const backdrop = document.getElementById("pane-nav-backdrop");
  const toggleBtn = document.getElementById("btn-toggle-nav");
  if (!paneNav) return;
  paneNav.classList.toggle("is-open", open);
  backdrop?.classList.toggle("is-open", open);
  toggleBtn?.setAttribute("aria-expanded", open ? "true" : "false");
}

function requestedHeadingId(): string | null {
  const id = readHash();
  return id && document.getElementById(id) ? id : null;
}

export function setupBookViewInteraction(): void {
  const toggleNavBtn = document.getElementById("btn-toggle-nav");
  const paneNav = document.getElementById("pane-nav");
  const navBackdrop = document.getElementById("pane-nav-backdrop");

  if (toggleNavBtn && paneNav) {
    toggleNavBtn.onclick = () => {
      const next = !paneNav.classList.contains("is-open");
      setMobileNavOpen(next);
    };
    if (navBackdrop) {
      navBackdrop.onclick = () => closeMobileNav();
    }
    paneNav.onclick = (e) => {
      if (
        window.innerWidth < 768 &&
        (e.target as Element | null)?.closest("a")
      ) {
        closeMobileNav();
      }
    };
  }

  if (!drawerEscapeBound) {
    drawerEscapeBound = true;
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        const nav = document.getElementById("pane-nav");
        if (nav?.classList.contains("is-open")) {
          closeMobileNav();
        }
      }
    });
  }

  activeSpy?.destroy();
  activeSpy = null;

  const scrollContainer = document.getElementById("book-content-scroll");
  if (!scrollContainer) return;

  const tocLinks = Array.from(
    document.querySelectorAll<HTMLElement>(".book-toc-link"),
  );

  let headingTargets: HeadingTarget[] = tocLinks
    .map((link): HeadingTarget | null => {
      const id = link.getAttribute("data-target");
      const el = id ? document.getElementById(id) : null;
      return id && el ? { id, el, link } : null;
    })
    .filter((x): x is HeadingTarget => x !== null);

  if (headingTargets.length === 0) {
    headingTargets = Array.from(
      scrollContainer.querySelectorAll<HTMLElement>(
        "h1[id], h2[id], h3[id], h4[id]",
      ),
    ).map((el) => ({ id: el.id, el, link: null }));
  }

  if (headingTargets.length === 0) return;

  const spy = createScrollSpy({
    container: scrollContainer,
    headingTargets,
    tocLinks,
    headingLine: 80,
    onActiveChange: (currentId) => {
      // Only write the heading hash after actual scrolling so initial load
      // stays at the top of the article instead of jumping past the title.
      if (scrollContainer.scrollTop > 0) {
        syncHeadingHash(currentId);
      }
    },
  });
  activeSpy = spy;

  for (const link of tocLinks) {
    link.onclick = (e) => {
      e.preventDefault();
      const targetId = link.getAttribute("data-target");
      const targetEl = targetId ? document.getElementById(targetId) : null;
      if (!targetId || !targetEl) return;

      spy.markClickScrolling(800);
      const top =
        scrollContainer.scrollTop +
        targetEl.getBoundingClientRect().top -
        scrollContainer.getBoundingClientRect().top -
        16;
      scrollContainer.scrollTo({ top, behavior: "smooth" });
      for (const l of tocLinks) l.classList.remove("is-active");
      link.classList.add("is-active");
      syncHeadingHash(targetId);
    };
  }

  const requested = requestedHeadingId();
  const settle = () => {
    if (requested) {
      const target = document.getElementById(requested);
      if (target) {
        const top =
          scrollContainer.scrollTop +
          target.getBoundingClientRect().top -
          scrollContainer.getBoundingClientRect().top -
          16;
        scrollContainer.scrollTo({ top, behavior: "auto" });
      }
    }
    spy.update();
  };
  settle();
  setTimeout(settle, 80);
}
