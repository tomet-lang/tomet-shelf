import type { Config } from "tailwindcss";

export default {
  content: ["./frontend/templates/**/*.html", "./frontend/src/**/*.ts"],
  darkMode: ["selector", '[data-theme="dark"]'],
  theme: {
    colors: {
      transparent: "transparent",
      current: "currentColor",
      canvas: "var(--bg-canvas)",
      bg: "var(--bg)",
      surface: "var(--bg-surface)",
      subtle: "var(--bg-subtle)",
      "surface-subtle": "var(--surface-subtle)",
      border: "var(--border)",
      "border-subtle": "var(--border-subtle)",
      text: "var(--text)",
      "text-primary": "var(--text-primary)",
      "text-secondary": "var(--text-secondary)",
      "text-muted": "var(--text-muted)",
      link: "var(--link)",
      "link-visited": "var(--link-visited)",
      unresolved: "var(--unresolved)",
      "accent-primary": "var(--accent-primary)",
      "shell-bg": "var(--shell-bg)",
      "shell-border": "var(--shell-border)",
      "shell-text": "var(--shell-text)",
      "shell-text-muted": "var(--shell-text-muted)",
    },
    boxShadow: {
      none: "var(--shadow-none)",
      sm: "var(--shadow-sm)",
      md: "var(--shadow-md)",
      lg: "var(--shadow-lg)",
      xl: "var(--shadow-xl)",
    },
  },
  corePlugins: {
    preflight: false,
    container: false,
  },
  plugins: [],
} satisfies Config;
