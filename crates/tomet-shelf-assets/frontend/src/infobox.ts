// Infobox image switcher and copy buttons (infobox cells & code blocks).

import { copyText } from "./dom";
import { t, withBasePath } from "./env";

const boundSections = new WeakSet<Element>();
const resetTimers = new WeakMap<HTMLElement, ReturnType<typeof setTimeout>>();
let copyDelegationBound = false;

export function setupInfoboxAndCopy(): void {
  document
    .querySelectorAll<HTMLElement>(".infobox-image-section")
    .forEach((section) => {
      if (boundSections.has(section)) return;
      boundSections.add(section);

      const img = section.querySelector<HTMLImageElement>(".infobox-main-img");
      const switcher = section.querySelector(".infobox-costume-switcher");
      const wrapper = section.querySelector<HTMLElement>(
        ".infobox-image-wrapper",
      );
      if (!img) return;

      const buttons = switcher
        ? Array.from(switcher.querySelectorAll<HTMLElement>(".costume-btn"))
        : [];

      const selectCostume = (index: number) => {
        if (buttons.length === 0) return;
        const targetBtn = buttons[(index + buttons.length) % buttons.length];
        if (!targetBtn || targetBtn.classList.contains("active")) return;
        const src = targetBtn.getAttribute("data-img-src");
        if (!src) return;

        img.style.opacity = "0.4";
        setTimeout(() => {
          img.src = src;
          const orig = targetBtn.getAttribute("data-original");
          if (orig) img.setAttribute("data-original", orig);
          else img.removeAttribute("data-original");
          img.style.opacity = "1";
        }, 80);

        for (const b of buttons) {
          b.classList.remove("active");
          b.removeAttribute("active");
        }
        targetBtn.classList.add("active");
        targetBtn.setAttribute("active", "");
      };

      const currentIndex = () => {
        const idx = buttons.findIndex((b) => b.classList.contains("active"));
        return idx >= 0 ? idx : 0;
      };

      buttons.forEach((btn, idx) => {
        btn.addEventListener("click", () => selectCostume(idx));
      });

      section
        .querySelector(".costume-arrow.prev")
        ?.addEventListener("click", (e) => {
          e.stopPropagation();
          selectCostume(currentIndex() - 1);
        });
      section
        .querySelector(".costume-arrow.next")
        ?.addEventListener("click", (e) => {
          e.stopPropagation();
          selectCostume(currentIndex() + 1);
        });

      if (wrapper && buttons.length > 1) {
        let startX = 0;
        let startY = 0;
        wrapper.addEventListener(
          "touchstart",
          (e) => {
            if (e.touches.length === 1) {
              startX = e.touches[0].clientX;
              startY = e.touches[0].clientY;
            }
          },
          { passive: true },
        );
        wrapper.addEventListener(
          "touchend",
          (e) => {
            if (e.changedTouches.length === 1) {
              const dx = e.changedTouches[0].clientX - startX;
              const dy = e.changedTouches[0].clientY - startY;
              if (Math.abs(dx) > 40 && Math.abs(dx) > Math.abs(dy) * 1.4) {
                selectCostume(currentIndex() + (dx > 0 ? -1 : 1));
              }
            }
          },
          { passive: true },
        );
      }
    });

  if (!copyDelegationBound) {
    copyDelegationBound = true;
    document.addEventListener("click", (e) => {
      const target = e.target as Element | null;
      if (!target) return;

      const codeBtn = target.closest<HTMLElement>(".code-copy-btn");
      if (codeBtn) {
        void handleCodeCopy(codeBtn);
        return;
      }

      const infoboxBtn = target.closest<HTMLElement>(".infobox-copy-btn");
      if (infoboxBtn) {
        e.preventDefault();
        e.stopPropagation();
        void handleInfoboxCopy(infoboxBtn);
      }
    });
  }
}

async function handleCodeCopy(btn: HTMLElement): Promise<void> {
  const wrapper = btn.closest(".code-block-wrapper");
  const code = wrapper
    ? wrapper.querySelector("pre code") || wrapper.querySelector("pre")
    : null;
  const text = code?.textContent || "";
  if (!(await copyText(text))) return;

  const use = btn.querySelector("use");
  if (use) use.setAttribute("href", withBasePath("/icons/lucide.svg#check"));
  const icon = btn.querySelector("tmt-icon");
  if (icon) icon.setAttribute("name", "check");

  const copiedLabel = t("code.copied", "コピーしました");
  btn.classList.add("is-copied");
  btn.setAttribute("title", copiedLabel);
  btn.setAttribute("aria-label", copiedLabel);

  const prev = resetTimers.get(btn);
  if (prev) clearTimeout(prev);
  resetTimers.set(
    btn,
    setTimeout(() => {
      if (use) use.setAttribute("href", withBasePath("/icons/lucide.svg#copy"));
      if (icon) icon.setAttribute("name", "copy");
      const copyLabel = t("code.copy", "コードをコピー");
      btn.classList.remove("is-copied");
      btn.setAttribute("title", copyLabel);
      btn.setAttribute("aria-label", copyLabel);
      resetTimers.delete(btn);
    }, 2000),
  );
}

async function handleInfoboxCopy(btn: HTMLElement): Promise<void> {
  const td = btn.closest(".infobox-data");
  if (!td) return;
  const valEl = td.querySelector(".infobox-data-val") || td;
  const clone = valEl.cloneNode(true) as HTMLElement;
  clone.querySelectorAll(".infobox-copy-btn").forEach((b) => b.remove());
  const text = (clone.textContent || "").trim();
  if (!(await copyText(text))) return;

  const originalHtml = btn.innerHTML;
  const copiedLabel = t("infobox.copied", "コピーしました");
  btn.classList.add("is-copied");
  btn.innerHTML =
    '<svg viewBox="0 0 24 24" width="12" height="12" stroke="#10b981" stroke-width="2.5" fill="none" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="20 6 9 17 4 12"></polyline></svg>';
  btn.setAttribute("title", copiedLabel);
  btn.setAttribute("aria-label", copiedLabel);

  const prev = resetTimers.get(btn);
  if (prev) clearTimeout(prev);
  resetTimers.set(
    btn,
    setTimeout(() => {
      const copyLabel = t("infobox.copy", "値をコピー");
      btn.classList.remove("is-copied");
      btn.innerHTML = originalHtml;
      btn.setAttribute("title", copyLabel);
      btn.setAttribute("aria-label", copyLabel);
      resetTimers.delete(btn);
    }, 1800),
  );
}
