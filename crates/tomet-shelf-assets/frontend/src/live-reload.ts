// Dev server WebSocket live reload (CSS cache-bust & scroll-preserving DOM swap).

import { fetchDocument, invalidateDocumentCache } from "./dom";
import { normalizePath } from "./env";
import { invalidatePreviewCache } from "./preview";
import { reinitPageContent, swapPageRegions } from "./router";

interface LiveReloadMessage {
  type?: string;
  url?: string;
}

function reloadStylesheets(): void {
  const ts = String(Date.now());
  document
    .querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]')
    .forEach((link) => {
      const url = new URL(link.href, window.location.href);
      url.searchParams.set("_tmt_ts", ts);
      link.href = url.toString();
    });
}

async function hotSwapCurrentDocument(): Promise<void> {
  try {
    const contentScroll = document.getElementById("book-content-scroll");
    const navScroll = document.querySelector<HTMLElement>(
      "#pane-nav .pane-scroll",
    );
    const prevContentScroll = contentScroll ? contentScroll.scrollTop : 0;
    const prevNavScroll = navScroll ? navScroll.scrollTop : 0;
    const prevWindowY = window.scrollY;

    const cleanUrl = window.location.href.split("#")[0];
    const newDoc = await fetchDocument(cleanUrl, true);
    if (!newDoc || !document.querySelector(".app-layout")) {
      window.location.reload();
      return;
    }

    swapPageRegions(newDoc, { cleanUrl, replaceNavTree: true });

    if (contentScroll) contentScroll.scrollTop = prevContentScroll;
    if (navScroll) navScroll.scrollTop = prevNavScroll;
    window.scrollTo(0, prevWindowY);

    reinitPageContent();
  } catch {
    window.location.reload();
  }
}

export function setupDevServerLiveReload(): void {
  if (
    window.location.protocol !== "http:" &&
    window.location.protocol !== "https:"
  ) {
    return;
  }

  try {
    const wsProtocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(
      `${wsProtocol}//${window.location.host}/live-reload`,
    );

    ws.onmessage = (event) => {
      let data: LiveReloadMessage | null = null;
      try {
        data = JSON.parse(String(event.data)) as LiveReloadMessage;
      } catch {
        if (event.data === "reload") data = { type: "full" };
      }
      if (!data) return;

      if (data.type === "css") {
        reloadStylesheets();
      } else if (data.type === "doc" && data.url) {
        invalidateDocumentCache();
        invalidatePreviewCache();
        const current = normalizePath(window.location.pathname).toLowerCase();
        const updated = normalizePath(data.url).toLowerCase();
        if (current === updated) {
          void hotSwapCurrentDocument();
        }
      } else {
        window.location.reload();
      }
    };
    ws.onerror = () => {};
  } catch {}
}
