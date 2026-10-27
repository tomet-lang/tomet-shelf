// Page-level configuration globals injected by base.html and page.html.

interface Window {
  tmtBasePath?: string;
  tmtUrlPrefix?: string;
  tmtFullUrlPrefix?: string;
  tmtHomeUrl?: string;
  tmtStrings?: Record<string, string>;
  tmtPageUrl?: string;
}

interface ViewTransition {
  finished: Promise<void>;
  ready: Promise<void>;
  updateCallbackDone: Promise<void>;
  skipTransition(): void;
}

interface Document {
  startViewTransition?(updateCallback: () => void | Promise<void>): ViewTransition;
}
