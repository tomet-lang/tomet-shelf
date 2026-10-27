// Declarative Shadow DOM Web Component primitives:
// <tmt-btn>, <tmt-badge>, <tmt-icon>, <tmt-swatch>, <tmt-switch>.

import { ensureShadowRoot } from "./dom";
import { isInternalNoteUrl, withBasePath } from "./env";
import { navigateTo } from "./router";

function openHref(href: string): void {
  if (isInternalNoteUrl(href)) {
    void navigateTo(href, true);
  } else {
    window.location.href = href;
  }
}

class TmtBtn extends HTMLElement {
  private innerBtn: HTMLButtonElement | null = null;
  private listenersBound = false;

  static get observedAttributes(): string[] {
    return ["active", "disabled", "title", "aria-label", "href", "class"];
  }

  connectedCallback(): void {
    ensureShadowRoot(this);
    this.innerBtn =
      this.shadowRoot?.querySelector<HTMLButtonElement>(".tmt-btn-inner") ??
      null;
    this.syncAttributes();

    if (!this.hasAttribute("tabindex") && !this.hasAttribute("disabled")) {
      this.setAttribute("tabindex", "0");
    }

    if (!this.listenersBound) {
      this.listenersBound = true;
      this.addEventListener("click", (e) => this.handleClick(e));
      this.addEventListener("keydown", (e) => this.handleKeyDown(e));
    }
  }

  attributeChangedCallback(
    _name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue !== newValue) this.syncAttributes();
  }

  private isActive(): boolean {
    return (
      this.hasAttribute("active") ||
      this.classList.contains("active") ||
      this.classList.contains("is-active")
    );
  }

  private handleClick(e: MouseEvent): void {
    if (this.hasAttribute("disabled")) {
      e.preventDefault();
      e.stopImmediatePropagation();
      return;
    }
    const href = this.getAttribute("href");
    if (href && !this.isActive() && !e.defaultPrevented) {
      openHref(href);
    }
  }

  private handleKeyDown(e: KeyboardEvent): void {
    if (this.hasAttribute("disabled")) return;
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      this.click();
    }
  }

  private syncAttributes(): void {
    if (!this.innerBtn) return;
    const disabled = this.hasAttribute("disabled");
    if (disabled) {
      this.setAttribute("aria-disabled", "true");
      this.setAttribute("tabindex", "-1");
      this.innerBtn.disabled = true;
    } else {
      this.removeAttribute("aria-disabled");
      if (this.getAttribute("tabindex") === "-1") {
        this.setAttribute("tabindex", "0");
      }
      this.innerBtn.disabled = false;
    }

    if (this.isActive()) {
      this.setAttribute("aria-pressed", "true");
    } else {
      this.removeAttribute("aria-pressed");
    }

    const title = this.getAttribute("title");
    const ariaLabel = this.getAttribute("aria-label");
    if (title) this.innerBtn.title = title;
    if (ariaLabel) this.innerBtn.setAttribute("aria-label", ariaLabel);
  }
}

class TmtBadge extends HTMLElement {
  private listenersBound = false;

  static get observedAttributes(): string[] {
    return ["href", "title", "aria-label"];
  }

  connectedCallback(): void {
    ensureShadowRoot(this);
    if (this.hasAttribute("href") && !this.hasAttribute("tabindex")) {
      this.setAttribute("tabindex", "0");
      this.setAttribute("role", "link");
    }
    if (!this.listenersBound) {
      this.listenersBound = true;
      this.addEventListener("click", (e) => this.handleClick(e));
      this.addEventListener("keydown", (e) => this.handleKeyDown(e));
    }
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue === newValue || name !== "href") return;
    if (newValue) {
      if (!this.hasAttribute("tabindex")) this.setAttribute("tabindex", "0");
      if (!this.hasAttribute("role")) this.setAttribute("role", "link");
    } else {
      if (this.getAttribute("role") === "link") this.removeAttribute("role");
      if (this.getAttribute("tabindex") === "0") {
        this.removeAttribute("tabindex");
      }
    }
    const innerLink = this.shadowRoot?.querySelector("a.tmt-badge-inner");
    if (innerLink) {
      if (newValue) innerLink.setAttribute("href", newValue);
      else innerLink.removeAttribute("href");
    }
  }

  private handleClick(e: MouseEvent): void {
    if (
      e.defaultPrevented ||
      e.metaKey ||
      e.ctrlKey ||
      e.shiftKey ||
      e.button !== 0
    ) {
      return;
    }
    const href = this.getAttribute("href");
    if (!href) return;
    e.preventDefault();
    openHref(href);
  }

  private handleKeyDown(e: KeyboardEvent): void {
    const href = this.getAttribute("href");
    if (href && e.key === "Enter") {
      e.preventDefault();
      openHref(href);
    }
  }
}

class TmtIcon extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["name", "size", "pkg"];
  }

  connectedCallback(): void {
    ensureShadowRoot(this);
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue === newValue) return;
    if ((name === "name" || name === "pkg") && this.shadowRoot) {
      const use = this.shadowRoot.querySelector("use");
      const iconName = this.getAttribute("name");
      const pkg = this.getAttribute("pkg") || "lucide";
      if (use && iconName) {
        use.setAttribute("href", withBasePath(`/icons/${pkg}.svg#${iconName}`));
      }
    }
  }
}

class TmtSwatch extends HTMLElement {
  private listenersBound = false;

  static get observedAttributes(): string[] {
    return ["checked", "disabled", "title"];
  }

  connectedCallback(): void {
    ensureShadowRoot(this);
    if (!this.hasAttribute("role")) this.setAttribute("role", "radio");
    this.syncAttributes();
    if (!this.listenersBound) {
      this.listenersBound = true;
      this.addEventListener("click", (e) => {
        if (this.hasAttribute("disabled")) e.preventDefault();
        else this.focus();
      });
      this.addEventListener("keydown", (e) => this.handleKeyDown(e));
    }
  }

  attributeChangedCallback(
    _name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue !== newValue) this.syncAttributes();
  }

  private siblings(): TmtSwatch[] {
    const group = this.closest('[role="radiogroup"]');
    return group
      ? Array.from(group.querySelectorAll<TmtSwatch>("tmt-swatch"))
      : [this];
  }

  private syncAttributes(): void {
    const checked = this.hasAttribute("checked");
    const disabled = this.hasAttribute("disabled");
    this.setAttribute("aria-checked", checked ? "true" : "false");
    if (disabled) {
      this.setAttribute("aria-disabled", "true");
      this.setAttribute("tabindex", "-1");
      return;
    }
    this.removeAttribute("aria-disabled");
    const siblings = this.siblings();
    const anyChecked = siblings.some((s) => s.hasAttribute("checked"));
    this.setAttribute(
      "tabindex",
      (anyChecked ? checked : siblings[0] === this) ? "0" : "-1",
    );
  }

  private handleKeyDown(e: KeyboardEvent): void {
    if (this.hasAttribute("disabled")) return;
    const moveKeys = [
      "ArrowUp",
      "ArrowDown",
      "ArrowLeft",
      "ArrowRight",
      "Home",
      "End",
    ];
    if (!moveKeys.includes(e.key)) return;
    e.preventDefault();

    const siblings = this.siblings().filter(
      (s) => !s.hasAttribute("disabled"),
    );
    if (siblings.length === 0) return;
    const idx = siblings.indexOf(this);

    let nextIdx: number;
    if (e.key === "Home") nextIdx = 0;
    else if (e.key === "End") nextIdx = siblings.length - 1;
    else if (e.key === "ArrowUp" || e.key === "ArrowLeft") {
      nextIdx = (idx - 1 + siblings.length) % siblings.length;
    } else {
      nextIdx = (idx + 1) % siblings.length;
    }

    const next = siblings[nextIdx];
    next.click();
    next.focus();
  }
}

class TmtSwitch extends HTMLElement {
  private listenersBound = false;

  static get observedAttributes(): string[] {
    return ["checked", "disabled", "title"];
  }

  connectedCallback(): void {
    ensureShadowRoot(this);
    if (!this.hasAttribute("role")) this.setAttribute("role", "switch");
    if (!this.hasAttribute("tabindex") && !this.hasAttribute("disabled")) {
      this.setAttribute("tabindex", "0");
    }
    this.syncAttributes();
    if (!this.listenersBound) {
      this.listenersBound = true;
      this.addEventListener("click", (e) => {
        if (this.hasAttribute("disabled")) e.preventDefault();
      });
      this.addEventListener("keydown", (e) => {
        if (this.hasAttribute("disabled")) return;
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          this.click();
        }
      });
    }
  }

  attributeChangedCallback(
    _name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue !== newValue) this.syncAttributes();
  }

  private syncAttributes(): void {
    const checked = this.hasAttribute("checked");
    const disabled = this.hasAttribute("disabled");
    this.setAttribute("aria-checked", checked ? "true" : "false");
    if (disabled) {
      this.setAttribute("aria-disabled", "true");
      this.setAttribute("tabindex", "-1");
    } else {
      this.removeAttribute("aria-disabled");
      if (this.getAttribute("tabindex") === "-1") {
        this.setAttribute("tabindex", "0");
      }
    }
  }
}

export function registerComponents(): void {
  if (!customElements.get("tmt-btn")) {
    customElements.define("tmt-btn", TmtBtn);
  }
  if (!customElements.get("tmt-badge")) {
    customElements.define("tmt-badge", TmtBadge);
  }
  if (!customElements.get("tmt-icon")) {
    customElements.define("tmt-icon", TmtIcon);
  }
  if (!customElements.get("tmt-swatch")) {
    customElements.define("tmt-swatch", TmtSwatch);
  }
  if (!customElements.get("tmt-switch")) {
    customElements.define("tmt-switch", TmtSwitch);
  }
}
