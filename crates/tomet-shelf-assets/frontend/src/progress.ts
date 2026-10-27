// Top page-load progress bar indicator.

let hideTimer: ReturnType<typeof setTimeout> | null = null;

export function startPageProgress(): void {
  const bar = document.getElementById("page-progress-bar");
  if (!bar) return;
  if (hideTimer) {
    clearTimeout(hideTimer);
    hideTimer = null;
  }
  bar.classList.remove("is-done", "is-loading");
  void bar.offsetWidth;
  bar.classList.add("is-loading");
}

export function finishPageProgress(): void {
  const bar = document.getElementById("page-progress-bar");
  if (!bar) return;
  bar.classList.remove("is-loading");
  bar.classList.add("is-done");
  if (hideTimer) clearTimeout(hideTimer);
  hideTimer = setTimeout(() => {
    bar.classList.remove("is-done");
    hideTimer = null;
  }, 450);
}
