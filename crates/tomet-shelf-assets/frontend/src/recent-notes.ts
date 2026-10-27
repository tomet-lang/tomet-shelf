// Visited note tracking and recent notes sidebar list.

import { storage } from "./dom";
import {
  isHomeOrRootPath,
  isInternalNoteUrl,
  normalizePath,
} from "./env";

interface RecentEntry {
  path: string;
  title: string;
}

const VISITED_KEY = "wiki-visited-notes";
const RECENT_KEY = "wiki-recent-notes";
const MAX_RECENT = 8;

let globalClickBound = false;

function readStringList(key: string): string[] {
  const raw = storage.get(key);
  if (!raw) return [];
  try {
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed)
      ? parsed.filter((x): x is string => typeof x === "string")
      : [];
  } catch {
    return [];
  }
}

function readRecentEntries(): RecentEntry[] {
  const raw = storage.get(RECENT_KEY);
  if (!raw) return [];
  try {
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter(
        (x): x is RecentEntry =>
          typeof x === "object" &&
          x !== null &&
          typeof (x as RecentEntry).path === "string" &&
          typeof (x as RecentEntry).title === "string",
      )
      .map((r) => ({ path: normalizePath(r.path), title: r.title }))
      .filter((r) => Boolean(r.path && r.title));
  } catch {
    return [];
  }
}

export function markVisited(url: string): void {
  const norm = normalizePath(url);
  if (!norm || isHomeOrRootPath(norm)) return;
  const set = new Set(readStringList(VISITED_KEY).map(normalizePath));
  set.add(norm);
  storage.set(VISITED_KEY, JSON.stringify([...set]));
}

export function syncVisitedClasses(): void {
  const list = readStringList(VISITED_KEY);
  if (list.length === 0) return;
  const visited = new Set(list.map(normalizePath));
  document
    .querySelectorAll<HTMLAnchorElement>(
      ".book-index-link, .search-result-item",
    )
    .forEach((a) => {
      const href = normalizePath(a.getAttribute("href"));
      if (href && visited.has(href)) {
        a.classList.add("is-visited");
      }
    });
}

export function updateRecentNotes(): void {
  if (!globalClickBound) {
    globalClickBound = true;
    document.addEventListener("click", (e) => {
      const a = (e.target as Element | null)?.closest("a");
      const href = a?.getAttribute("href");
      if (a && isInternalNoteUrl(href)) {
        markVisited(href);
        a.classList.add("is-visited");
      }
    });
  }

  const currentPath = normalizePath(window.location.pathname);
  const titleEl =
    document.querySelector(".content-title") ||
    document.querySelector(".article-header h1");
  const currentTitle =
    titleEl?.textContent?.trim() || document.title.replace(/\s*\|.*$/, "");

  if (currentPath && currentTitle && !isHomeOrRootPath(currentPath)) {
    markVisited(currentPath);

    const recents = readRecentEntries().filter((r) => r.path !== currentPath);
    recents.unshift({ path: currentPath, title: currentTitle });
    const trimmed = recents.slice(0, MAX_RECENT);
    storage.set(RECENT_KEY, JSON.stringify(trimmed));
  }

  renderRecentNotesList(currentPath);
  syncVisitedClasses();
}

function renderRecentNotesList(currentPath: string): void {
  const recentSection = document.getElementById("pane-recent-section");
  const recentList = document.getElementById("pane-recent-list");
  const itemTemplate = document.getElementById(
    "recent-note-item-template",
  ) as HTMLTemplateElement | null;
  if (!recentSection || !recentList || !itemTemplate) return;

  const recents = readRecentEntries();
  if (recents.length === 0) {
    recentSection.hidden = true;
    return;
  }

  const items: HTMLElement[] = [];
  for (const entry of recents) {
    const li = itemTemplate.content.firstElementChild?.cloneNode(true) as
      | HTMLElement
      | undefined;
    if (!li) continue;
    const link = li.querySelector<HTMLAnchorElement>(".recent-link");
    if (link) {
      const isCurrent = entry.path === currentPath;
      link.href = entry.path;
      link.classList.toggle("is-current", isCurrent);
      link.classList.toggle("font-semibold", isCurrent);
      link.classList.toggle("text-accent-primary", isCurrent);
      link.classList.toggle("bg-subtle", isCurrent);
      if (isCurrent) link.setAttribute("aria-current", "page");
      const titleSpan = link.querySelector(".recent-title");
      if (titleSpan) titleSpan.textContent = entry.title;
    }
    items.push(li);
  }

  recentList.replaceChildren(...items);
  recentSection.hidden = items.length === 0;
}
