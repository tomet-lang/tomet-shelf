// Incremental client-side search (Native JSON index + Pagefind fallback).

import { isInternalNoteUrl, t, urlPrefix, withBasePath } from "./env";
import { navigateTo } from "./router";

interface NativeSearchDocument {
  id: number;
  slug: string;
  title: string;
  url: string;
  path: string;
  aliases?: string[];
  kind?: string;
  section?: string;
  image?: string;
  headings?: string[];
  text: string;
  _metaHaystack?: string;
  _textLower?: string;
  _fullHaystack?: string;
  _targets?: NormalizedTarget[];
}

interface NativeSearchIndex {
  docs: NativeSearchDocument[];
}

interface PagefindSubResult {
  score: number;
  data(): Promise<{
    url: string;
    excerpt?: string;
    meta?: Record<string, string | undefined>;
    filters?: Record<string, string | string[] | undefined>;
  }>;
}

interface PagefindApi {
  options(opts: Record<string, unknown>): Promise<void>;
  init(): Promise<void>;
  search(query: string): Promise<{ results: PagefindSubResult[] } | null>;
}

interface NormalizedTarget {
  strict: string;
  fuzzy: string;
  weight: number;
}

interface CompiledQuery {
  strict: string;
  fuzzy: string;
  wordPattern: RegExp | null;
}

interface SearchCandidate {
  url: string;
  rawScore: number;
  boost: number;
  excerpt?: string;
  meta: {
    title?: string;
    path?: string;
    aliases?: string;
    image?: string;
  };
  filters: {
    kind?: string;
    section?: string;
  };
}

let pagefindInstance: PagefindApi | null = null;
let nativeIndexPromise: Promise<NativeSearchIndex | null> | null = null;
let globalShortcutBound = false;

async function getPagefind(): Promise<PagefindApi | null> {
  if (!pagefindInstance) {
    try {
      const pagefindUrl = withBasePath("/pagefind/pagefind.js");
      const pf = (await import(
        /* @vite-ignore */ pagefindUrl
      )) as PagefindApi;
      await pf.options({ ranking: { pageLength: 0.1 } });
      await pf.init();
      pagefindInstance = pf;
    } catch {
      return null;
    }
  }
  return pagefindInstance;
}

function getNativeIndex(): Promise<NativeSearchIndex | null> {
  if (!nativeIndexPromise) {
    nativeIndexPromise = fetch(withBasePath("/search-index.json"))
      .then((res) => (res.ok ? (res.json() as Promise<NativeSearchIndex>) : null))
      .catch(() => null);
  }
  return nativeIndexPromise;
}

function tokenizeQuery(query: string): string[] {
  const q = query.trim();
  if (!q) return [];

  if (typeof Intl !== "undefined" && typeof Intl.Segmenter === "function") {
    const segmenter = new Intl.Segmenter(undefined, { granularity: "word" });
    const tokens: string[] = [];
    for (const { segment, isWordLike } of segmenter.segment(q)) {
      if (isWordLike) tokens.push(segment.toLowerCase());
    }
    if (tokens.length > 0) return tokens;
  }

  return q
    .toLowerCase()
    .split(/[\s、,，.。]+/)
    .filter(Boolean);
}

function normalizeStrict(s: string): string {
  return s.toLowerCase().trim();
}

function normalizeFuzzy(s: string): string {
  return s
    .toLowerCase()
    .replace(/[@#\-_/\\.()[\]{}:;,+*~]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function compileQuery(query: string): CompiledQuery {
  const strict = normalizeStrict(query);
  const fuzzy = normalizeFuzzy(query);
  const wordPattern = fuzzy
    ? new RegExp(
        `(^|\\s)${fuzzy.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&")}($|\\s)`,
      )
    : null;
  return { strict, fuzzy, wordPattern };
}

function buildTargets(
  title: string,
  path: string,
  url: string,
  aliases: string[],
): NormalizedTarget[] {
  const filename = path
    ? path
        .split("/")
        .pop()
        ?.replace(/\.tmt$/, "") || ""
    : "";
  const urlStem = url.replace(/\/+$/, "").split("/").pop() || "";

  const raw: Array<{ text: string; weight: number }> = [
    { text: title, weight: 1.0 },
    { text: filename, weight: 1.0 },
    { text: urlStem, weight: 0.9 },
    { text: path, weight: 0.8 },
    ...aliases.map((a) => ({ text: a, weight: 0.98 })),
  ];

  return raw
    .filter((entry) => Boolean(entry.text))
    .map((entry) => ({
      strict: normalizeStrict(entry.text),
      fuzzy: normalizeFuzzy(entry.text),
      weight: entry.weight,
    }));
}

/**
 * 4-tier boost ranking shared across Native Search and Pagefind fallback:
 * Exact (100k) > Prefix (50k) > Word-boundary (20k) > Substring (10k).
 */
function scoreTargets(
  compiled: CompiledQuery,
  targets: NormalizedTarget[],
): number {
  if (!compiled.strict) return 0;
  let maxBoost = 0;

  for (const { strict, fuzzy, weight } of targets) {
    if (strict === compiled.strict || (compiled.fuzzy && fuzzy === compiled.fuzzy)) {
      maxBoost = Math.max(maxBoost, 100000 * weight);
    } else if (
      strict.startsWith(compiled.strict) ||
      (compiled.fuzzy && fuzzy.startsWith(compiled.fuzzy))
    ) {
      maxBoost = Math.max(maxBoost, 50000 * weight);
    } else if (compiled.wordPattern?.test(fuzzy)) {
      maxBoost = Math.max(maxBoost, 20000 * weight);
    } else if (
      strict.includes(compiled.strict) ||
      (compiled.fuzzy && fuzzy.includes(compiled.fuzzy))
    ) {
      maxBoost = Math.max(maxBoost, 10000 * weight);
    }
  }

  return maxBoost;
}

function getDocTargets(doc: NativeSearchDocument): NormalizedTarget[] {
  if (!doc._targets) {
    doc._targets = buildTargets(
      doc.title || "",
      doc.path || "",
      doc.url || "",
      doc.aliases || [],
    );
  }
  return doc._targets;
}

function getDocHaystack(
  doc: NativeSearchDocument,
  includeFullText: boolean,
): string {
  if (doc._metaHaystack === undefined) {
    doc._metaHaystack = [
      doc.title,
      ...(doc.aliases || []),
      ...(doc.headings || []),
      doc.path,
    ]
      .join(" ")
      .toLowerCase();
  }
  if (!includeFullText) return doc._metaHaystack;
  if (doc._fullHaystack === undefined) {
    doc._textLower ??= (doc.text || "").toLowerCase();
    doc._fullHaystack = `${doc._metaHaystack} ${doc._textLower}`;
  }
  return doc._fullHaystack;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function buildHighlightedExcerpt(
  text: string,
  tokens: string[],
  maxLen = 160,
): string {
  if (!text) return "";
  const lowerText = text.toLowerCase();

  let matchIndex = -1;
  for (const tok of tokens) {
    const idx = lowerText.indexOf(tok);
    if (idx !== -1 && (matchIndex === -1 || idx < matchIndex)) {
      matchIndex = idx;
    }
  }

  const anchor = matchIndex === -1 ? 0 : matchIndex;
  const start = Math.max(0, anchor - Math.floor(maxLen / 3));
  const end = Math.min(text.length, start + maxLen);
  const slice = text.slice(start, end);

  const uniqueTokens = Array.from(new Set(tokens.filter(Boolean))).sort(
    (a, b) => b.length - a.length,
  );
  let highlighted = escapeHtml(slice);
  if (uniqueTokens.length > 0) {
    const pattern = uniqueTokens
      .map((tok) => tok.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&"))
      .join("|");
    highlighted = highlighted.replace(
      new RegExp(`(${pattern})`, "gi"),
      "<mark>$1</mark>",
    );
  }

  return `${start > 0 ? "…" : ""}${highlighted}${end < text.length ? "…" : ""}`;
}

function searchNativeIndex(
  query: string,
  index: NativeSearchIndex,
): SearchCandidate[] {
  const trimmed = query.trim();
  const tokens = tokenizeQuery(trimmed);
  if (tokens.length === 0) return [];

  const searchFullText = trimmed.length >= 2;
  const compiled = compileQuery(trimmed);
  const sortedTokens = [...tokens].sort((a, b) => b.length - a.length);

  const matches: SearchCandidate[] = [];
  for (const doc of index.docs) {
    const haystack = getDocHaystack(doc, searchFullText);
    if (!sortedTokens.every((tok) => haystack.includes(tok))) continue;

    const boost = scoreTargets(compiled, getDocTargets(doc));
    let rawScore = 0;
    if (searchFullText && doc.text) {
      doc._textLower ??= doc.text.toLowerCase();
      for (const tok of tokens) {
        let from = 0;
        for (;;) {
          const idx = doc._textLower.indexOf(tok, from);
          if (idx === -1) break;
          rawScore += 1;
          from = idx + tok.length;
        }
      }
    }

    let computedExcerpt: string | undefined;
    let hasComputed = false;
    matches.push({
      url: doc.url,
      boost,
      rawScore,
      meta: {
        title: doc.title,
        path: doc.path,
        aliases: (doc.aliases || []).join(", "),
        image: doc.image,
      },
      filters: {
        kind: doc.kind,
        section: doc.section,
      },
      get excerpt(): string | undefined {
        if (!hasComputed) {
          hasComputed = true;
          computedExcerpt = doc.text
            ? buildHighlightedExcerpt(doc.text, tokens)
            : "";
        }
        return computedExcerpt;
      },
    });
  }

  matches.sort((a, b) => b.boost - a.boost || b.rawScore - a.rawScore);
  return matches;
}

async function searchPagefindFallback(
  query: string,
): Promise<SearchCandidate[] | null> {
  const pf = await getPagefind();
  if (!pf) return null;

  const search = await pf.search(query);
  if (!search || search.results.length === 0) return [];

  const compiled = compileQuery(query);
  const slice = search.results.slice(0, 30);
  const candidates = await Promise.all(
    slice.map(async (r): Promise<SearchCandidate> => {
      const data = await r.data();
      const title = data.meta?.title || "";
      const path = data.meta?.path || "";
      const aliasesStr = data.meta?.aliases || "";
      const aliasList = aliasesStr
        ? aliasesStr
            .split(/[,、]/)
            .map((s) => s.trim())
            .filter(Boolean)
        : [];
      const firstFilter = (val: string | string[] | undefined) =>
        Array.isArray(val) ? val[0] : val;

      return {
        url: data.url,
        rawScore: r.score,
        boost: scoreTargets(
          compiled,
          buildTargets(title, path, data.url, aliasList),
        ),
        excerpt: data.excerpt,
        meta: {
          title: data.meta?.title,
          path: data.meta?.path,
          aliases: data.meta?.aliases,
          image: data.meta?.image,
        },
        filters: {
          kind: firstFilter(data.filters?.kind),
          section: firstFilter(data.filters?.section),
        },
      };
    }),
  );

  candidates.sort((a, b) => b.boost - a.boost || b.rawScore - a.rawScore);
  return candidates;
}

export function setupSearch(): void {
  if (!globalShortcutBound) {
    globalShortcutBound = true;
    window.addEventListener("keydown", (e) => {
      const target = e.target as HTMLElement | null;
      const isEditing =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable);

      if (
        ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") ||
        (e.key === "/" && !isEditing)
      ) {
        const input = document.getElementById(
          "wiki-search-input",
        ) as HTMLInputElement | null;
        if (input) {
          e.preventDefault();
          input.focus();
          input.select();
        }
      }
    });
  }

  const input = document.getElementById(
    "wiki-search-input",
  ) as HTMLInputElement | null;
  const resultsEl = document.getElementById("wiki-search-results");
  if (!input || !resultsEl) return;
  if (input.dataset.searchInitialized === "true") return;
  input.dataset.searchInitialized = "true";

  const shortcutBadge = document.querySelector<HTMLElement>(
    ".search-shortcut-badge",
  );
  shortcutBadge?.addEventListener("click", (e) => {
    e.preventDefault();
    input.focus();
    input.select();
  });

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let selectedIndex = -1;
  let searchSeq = 0;

  const showStatus = (message: string) => {
    const status = document.createElement("div");
    status.className = "search-status";
    status.textContent = message;
    resultsEl.replaceChildren(status);
    resultsEl.hidden = false;
  };

  const renderResults = (results: SearchCandidate[]) => {
    const tpl = document.getElementById(
      "search-result-item-template",
    ) as HTMLTemplateElement | null;
    if (!tpl) return;

    const cleanPrefixRe = new RegExp(
      `^${urlPrefix().replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}/?`,
    );

    const items: HTMLElement[] = [];
    results.forEach((res, idx) => {
      const item = tpl.content.firstElementChild?.cloneNode(true) as
        | HTMLAnchorElement
        | undefined;
      if (!item) return;

      item.href = res.url;
      item.dataset.index = String(idx);

      const thumb = item.querySelector<HTMLImageElement>(".search-result-thumb");
      if (res.meta.image && thumb) {
        thumb.src = res.meta.image;
        thumb.hidden = false;
        thumb.addEventListener("error", () => thumb.remove(), { once: true });
      } else {
        thumb?.remove();
      }

      const titleText = item.querySelector(".result-title-text");
      if (titleText) titleText.textContent = res.meta.title || res.url;

      const kindEl = item.querySelector<HTMLElement>(".search-result-kind");
      if (res.filters.kind && kindEl) {
        kindEl.textContent = res.filters.kind;
        kindEl.hidden = false;
      } else {
        kindEl?.remove();
      }

      const sectionEl = item.querySelector<HTMLElement>(
        ".search-result-section",
      );
      if (res.filters.section && sectionEl) {
        sectionEl.textContent = res.filters.section;
        sectionEl.hidden = false;
      } else {
        sectionEl?.remove();
      }

      const displayPath =
        res.meta.path ||
        res.url.replace(cleanPrefixRe, "").replace(/\/+$/, "");
      const pathEl = item.querySelector<HTMLElement>(".search-result-path");
      if (displayPath && pathEl) {
        pathEl.textContent = displayPath;
        pathEl.hidden = false;
      } else {
        pathEl?.remove();
      }

      const aliasesEl = item.querySelector<HTMLElement>(
        ".search-result-aliases",
      );
      if (res.meta.aliases && aliasesEl) {
        const label = aliasesEl.querySelector(".alias-label");
        const text = aliasesEl.querySelector(".alias-text");
        if (label) label.textContent = t("search.aliases", "別名:");
        if (text) text.textContent = res.meta.aliases;
        aliasesEl.hidden = false;
      } else {
        aliasesEl?.remove();
      }

      const excerptEl = item.querySelector<HTMLElement>(
        ".search-result-excerpt",
      );
      const excerptHtml = res.excerpt;
      if (excerptHtml && excerptEl) {
        const plainExcerpt = excerptHtml.replace(/<[^>]+>/g, "").trim();
        const plainPath = displayPath.trim();
        if (
          plainExcerpt &&
          plainPath &&
          (plainExcerpt === plainPath || plainExcerpt.startsWith(plainPath))
        ) {
          excerptEl.remove();
        } else {
          excerptEl.innerHTML = excerptHtml;
          excerptEl.hidden = false;
        }
      } else {
        excerptEl?.remove();
      }

      items.push(item);
    });

    resultsEl.replaceChildren(...items);
    resultsEl.hidden = false;
  };

  const updateSelection = (items: NodeListOf<HTMLAnchorElement>) => {
    items.forEach((item, idx) => {
      if (idx === selectedIndex) {
        item.classList.add("selected");
        item.scrollIntoView({ block: "nearest" });
      } else {
        item.classList.remove("selected");
      }
    });
  };

  input.addEventListener("focus", () => {
    void getNativeIndex();
    if (input.value.trim().length > 0 && resultsEl.children.length > 0) {
      resultsEl.hidden = false;
    }
  });

  input.addEventListener("input", () => {
    if (debounceTimer) clearTimeout(debounceTimer);
    const query = input.value.trim();
    if (!query) {
      resultsEl.replaceChildren();
      resultsEl.hidden = true;
      selectedIndex = -1;
      return;
    }

    const seq = ++searchSeq;
    debounceTimer = setTimeout(async () => {
      const nativeIndex = await getNativeIndex();
      if (seq !== searchSeq) return;

      if (nativeIndex) {
        const top = searchNativeIndex(query, nativeIndex).slice(0, 10);
        selectedIndex = -1;
        if (top.length === 0) {
          showStatus(t("search.empty", "一致するノートが見つかりません"));
        } else {
          renderResults(top);
        }
        return;
      }

      const fallback = await searchPagefindFallback(query);
      if (seq !== searchSeq) return;
      selectedIndex = -1;

      if (fallback === null) {
        showStatus(t("search.loading", "検索インデックスを読み込み中..."));
      } else if (fallback.length === 0) {
        showStatus(t("search.empty", "一致するノートが見つかりません"));
      } else {
        renderResults(fallback.slice(0, 10));
      }
    }, 160);
  });

  input.addEventListener("keydown", (e) => {
    const items =
      resultsEl.querySelectorAll<HTMLAnchorElement>(".search-result-item");
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (items.length > 0) {
        selectedIndex = (selectedIndex + 1) % items.length;
        updateSelection(items);
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (items.length > 0) {
        selectedIndex = (selectedIndex - 1 + items.length) % items.length;
        updateSelection(items);
      }
    } else if (e.key === "Enter") {
      const target =
        selectedIndex >= 0 && items[selectedIndex]
          ? items[selectedIndex]
          : items[0];
      if (target) {
        e.preventDefault();
        const href = target.getAttribute("href");
        resultsEl.hidden = true;
        input.blur();
        if (href && isInternalNoteUrl(href)) {
          void navigateTo(href, true);
        } else if (href) {
          window.location.href = href;
        }
      }
    } else if (e.key === "Escape") {
      resultsEl.hidden = true;
      input.blur();
    }
  });

  resultsEl.addEventListener("click", (e) => {
    if ((e.target as Element | null)?.closest("a")) {
      resultsEl.hidden = true;
      input.blur();
    }
  });

  document.addEventListener("click", (e) => {
    const targetNode = e.target as Node | null;
    if (
      !input.contains(targetNode) &&
      !resultsEl.contains(targetNode) &&
      !shortcutBadge?.contains(targetNode)
    ) {
      resultsEl.hidden = true;
    }
  });
}
