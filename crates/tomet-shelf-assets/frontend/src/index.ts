// tomet-shelf frontend runtime entry point.

import { initCenterPeek } from "./center-peek";
import { registerComponents } from "./components";
import { setupDevServerLiveReload } from "./live-reload";
import { initPagePreview } from "./preview";
import { reinitPageContent, setupClientRouter } from "./router";
import { setupSearch } from "./search";
import { setupThemeToggle } from "./theme";

function initApp(): void {
  setupThemeToggle();
  setupSearch();
  initPagePreview();
  initCenterPeek();
  reinitPageContent();
  setupClientRouter();
  setupDevServerLiveReload();
}

registerComponents();

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initApp);
} else {
  initApp();
}
