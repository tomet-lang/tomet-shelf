// Dev-server authoring helpers: edit dropdown menu, toast, and read-only source viewer.

import { copyText } from "./dom";
import { t } from "./env";

let toastTimer: ReturnType<typeof setTimeout> | null = null;
let outsideClickBound = false;
let sourceOverlay: HTMLElement | null = null;

export function showToast(message: string): void {
  let toast = document.getElementById("tmt-global-toast");
  if (!toast) {
    toast = document.createElement("div");
    toast.id = "tmt-global-toast";
    toast.className = "tmt-toast";
    document.body.appendChild(toast);
  }
  toast.textContent = message;
  toast.classList.add("is-show");
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast?.classList.remove("is-show");
    toastTimer = null;
  }, 2500);
}

function closeAllEditDropdowns(): void {
  document.querySelectorAll(".edit-dropdown.is-open").forEach((d) => {
    d.classList.remove("is-open");
    const menu = d.querySelector<HTMLElement>(".edit-dropdown-menu");
    if (menu) menu.hidden = true;
  });
}

function closeSourceOverlay(): void {
  sourceOverlay?.remove();
  sourceOverlay = null;
  document.removeEventListener("keydown", onSourceKeydown, true);
}

function onSourceKeydown(e: KeyboardEvent): void {
  if (sourceOverlay && e.key === "Escape") {
    e.preventDefault();
    closeSourceOverlay();
  }
}

function openSourceOverlay(source: string): void {
  closeSourceOverlay();

  const overlay = document.createElement("div");
  overlay.className = "tmt-source-viewer-overlay";

  const panel = document.createElement("div");
  panel.className = "tmt-source-viewer-panel";

  const toolbar = document.createElement("div");
  toolbar.className = "tmt-source-viewer-toolbar";

  const closeBtn = document.createElement("button");
  closeBtn.type = "button";
  closeBtn.className = "tmt-source-viewer-btn";
  closeBtn.textContent = t("edit.close", "閉じる");
  closeBtn.onclick = closeSourceOverlay;
  toolbar.appendChild(closeBtn);

  const textarea = document.createElement("textarea");
  textarea.className = "tmt-source-viewer-textarea";
  textarea.value = source;
  textarea.readOnly = true;
  textarea.spellcheck = false;

  panel.append(toolbar, textarea);
  overlay.appendChild(panel);
  document.body.appendChild(overlay);
  sourceOverlay = overlay;

  overlay.addEventListener("mousedown", (e) => {
    if (e.target === overlay) closeSourceOverlay();
  });
  document.addEventListener("keydown", onSourceKeydown, true);

  textarea.focus();
  textarea.setSelectionRange(0, 0);
}

async function fetchSource(path: string): Promise<string | null> {
  try {
    const res = await fetch(
      `/__shelf/source?path=${encodeURIComponent(path)}`,
      { cache: "no-store" },
    );
    return res.ok ? await res.text() : null;
  } catch {
    return null;
  }
}

export function setupDevTools(): void {
  const editBtns = document.querySelectorAll<HTMLElement>(".btn-edit-open");
  if (editBtns.length === 0) return;

  editBtns.forEach((btn) => {
    const dropdown = btn.closest<HTMLElement>(".edit-dropdown");
    const menu = dropdown?.querySelector<HTMLElement>(".edit-dropdown-menu");
    if (!dropdown || !menu) return;

    btn.onclick = (e) => {
      e.preventDefault();
      e.stopPropagation();
      const wasOpen = dropdown.classList.contains("is-open");
      closeAllEditDropdowns();
      if (!wasOpen) {
        dropdown.classList.add("is-open");
        menu.hidden = false;
      }
    };

    let hoverTimer: ReturnType<typeof setTimeout> | null = null;
    dropdown.onmouseenter = () => {
      if (window.matchMedia("(hover: hover)").matches) {
        if (hoverTimer) clearTimeout(hoverTimer);
        dropdown.classList.add("is-open");
        menu.hidden = false;
      }
    };
    dropdown.onmouseleave = () => {
      if (window.matchMedia("(hover: hover)").matches) {
        hoverTimer = setTimeout(() => {
          dropdown.classList.remove("is-open");
          menu.hidden = true;
        }, 180);
      }
    };
  });

  document.querySelectorAll<HTMLElement>(".btn-copy-path").forEach((btn) => {
    btn.onclick = async (e) => {
      e.preventDefault();
      e.stopPropagation();
      const path = btn.getAttribute("data-path") || "";
      if (path) {
        const ok = await copyText(path);
        showToast(
          ok
            ? t("toast.copied", "パスをコピーしました")
            : t("toast.copy_failed", "コピーに失敗しました"),
        );
      }
      closeAllEditDropdowns();
    };
  });

  document
    .querySelectorAll<HTMLButtonElement>(".btn-source-view-toggle")
    .forEach((toggle) => {
      toggle.onclick = async (e) => {
        e.preventDefault();
        e.stopPropagation();
        closeAllEditDropdowns();
        const path = toggle.dataset.sourcePath || "";
        if (!path) return;
        const source = await fetchSource(path);
        if (source === null) {
          showToast(
            t("toast.source_load_failed", "ソースの読み込みに失敗しました"),
          );
          return;
        }
        openSourceOverlay(source);
      };
    });

  if (!outsideClickBound) {
    outsideClickBound = true;
    document.addEventListener("click", (e) => {
      if (!(e.target as Element | null)?.closest(".edit-dropdown")) {
        closeAllEditDropdowns();
      }
    });
  }
}
