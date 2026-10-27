// Image zoom lightbox modal.

const boundTargets = new WeakSet<Element>();
let overlayBound = false;

export function openLightbox(src: string, alt = ""): void {
  const overlay = document.getElementById("wiki-image-lightbox");
  const img = document.getElementById(
    "lightbox-img",
  ) as HTMLImageElement | null;
  const caption = document.getElementById("lightbox-caption");
  if (!overlay || !img) return;

  img.src = src;
  img.alt = alt;
  if (caption) caption.textContent = alt;
  overlay.hidden = false;
}

export function closeLightbox(): void {
  const overlay = document.getElementById("wiki-image-lightbox");
  const img = document.getElementById(
    "lightbox-img",
  ) as HTMLImageElement | null;
  if (!overlay || overlay.hidden) return;

  overlay.hidden = true;
  if (img) img.src = "";
}

export function setupImageLightbox(): void {
  const overlay = document.getElementById("wiki-image-lightbox");
  if (!overlay) return;

  if (!overlayBound) {
    overlayBound = true;
    overlay
      .querySelector(".lightbox-close")
      ?.addEventListener("click", closeLightbox);

    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) closeLightbox();
    });

    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !overlay.hidden) closeLightbox();
    });

    let touchStartX = 0;
    let touchStartY = 0;
    overlay.addEventListener(
      "touchstart",
      (e) => {
        if (e.touches.length === 1) {
          touchStartX = e.touches[0].clientX;
          touchStartY = e.touches[0].clientY;
        }
      },
      { passive: true },
    );
    overlay.addEventListener(
      "touchend",
      (e) => {
        if (e.changedTouches.length === 1) {
          const deltaY = e.changedTouches[0].clientY - touchStartY;
          const deltaX = e.changedTouches[0].clientX - touchStartX;
          if (deltaY > 60 && deltaY > Math.abs(deltaX) * 1.5) {
            closeLightbox();
          }
        }
      },
      { passive: true },
    );
  }

  document
    .querySelectorAll<HTMLImageElement>(
      "article img:not(.tm-link-icon):not(.tm-doc-icon-img), .infobox-main-img, .hero-banner-img",
    )
    .forEach((img) => {
      if (boundTargets.has(img)) return;
      boundTargets.add(img);
      img.addEventListener("click", () => {
        const src =
          img.getAttribute("data-original") || img.getAttribute("src");
        if (src) openLightbox(src, img.getAttribute("alt") || "");
      });
    });

  document.querySelectorAll<HTMLElement>(".hero-zoom-btn").forEach((btn) => {
    if (boundTargets.has(btn)) return;
    boundTargets.add(btn);
    btn.addEventListener("click", () => {
      const wrapper = btn.closest(".hero-banner-wrapper");
      const img =
        wrapper?.querySelector(".hero-banner-img") ??
        document.querySelector(".hero-banner-img");
      const src =
        img?.getAttribute("data-original") || img?.getAttribute("src");
      if (src) openLightbox(src, img?.getAttribute("alt") || "Banner");
    });
  });
}
