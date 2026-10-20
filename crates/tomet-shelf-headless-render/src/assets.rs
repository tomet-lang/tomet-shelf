use anyhow::{Context, Result};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::LazyLock;

use tomet_shelf_config::BookConfig;

pub const LUCIDE_SPRITE: &str = include_str!("../assets/icons/lucide/sprite.svg");
pub const SIMPLE_SPRITE: &str = include_str!("../assets/icons/simple/sprite.svg");

pub const TMT_BTN_CSS: &str = include_str!("../assets/components/tmt-btn.css");
pub const TMT_BADGE_CSS: &str = include_str!("../assets/components/tmt-badge.css");
pub const TMT_ICON_CSS: &str = include_str!("../assets/components/tmt-icon.css");
pub const TMT_SWATCH_CSS: &str = include_str!("../assets/components/tmt-swatch.css");
pub const TMT_SWITCH_CSS: &str = include_str!("../assets/components/tmt-switch.css");

pub fn content_version(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub static TMT_BTN_CSS_VERSION: LazyLock<String> = LazyLock::new(|| content_version(TMT_BTN_CSS));
pub static TMT_BADGE_CSS_VERSION: LazyLock<String> =
    LazyLock::new(|| content_version(TMT_BADGE_CSS));
pub static TMT_ICON_CSS_VERSION: LazyLock<String> = LazyLock::new(|| content_version(TMT_ICON_CSS));
pub static TMT_SWATCH_CSS_VERSION: LazyLock<String> =
    LazyLock::new(|| content_version(TMT_SWATCH_CSS));
pub static TMT_SWITCH_CSS_VERSION: LazyLock<String> =
    LazyLock::new(|| content_version(TMT_SWITCH_CSS));

/// Minimal built-in templates used when no custom UI assets are supplied (e.g. headless tests).
pub const MINIMAL_BASE_HTML: &str = r#"<!DOCTYPE html>
<html lang="{{ lang | default('ja') }}" data-view="{{ default_view | default('book') }}">
<head>
  <meta charset="utf-8" />
  <title>{% if page_title %}{{ page_title }} | {% endif %}{{ site_title }}</title>
  <link rel="stylesheet" href="{{ base_path | safe }}/shelf.css?v={{ css_version }}" />
  <link rel="stylesheet" href="{{ base_path | safe }}/components/tmt-btn.css?v={{ tmt_btn_css_version }}" />
  {% for css in custom_css %}<link rel="stylesheet" href="{{ css }}" />{% endfor %}
</head>
<body>
  <a href="{{ home_url | safe }}">{{ site_title }}</a>
  {% block content %}{% endblock %}
  <script>
    window.tmtBasePath = {{ base_path | tojson }};
    window.tmtUrlPrefix = {{ clean_url_prefix | tojson }};
    window.tmtFullUrlPrefix = {{ full_url_prefix | tojson }};
    window.tmtHomeUrl = {{ home_url | tojson }};
    window.tmtStrings = {{ t | tojson | safe }};
  </script>
  <script src="{{ base_path | safe }}/shelf.js?v={{ js_version }}"></script>
</body>
</html>"#;

pub const MINIMAL_PAGE_HTML: &str = r#"{% extends "base.html" %}
{% block content %}
<main data-pagefind-body>
  {% if slug %}<span class="sr-only" data-pagefind-meta="path" data-pagefind-weight="10.0">{{ slug | safe }}</span>{% endif %}
  {% if rel_path and rel_path != slug %}<span class="sr-only" data-pagefind-weight="10.0">{{ rel_path | safe }}</span>{% endif %}
  {% if icon %}<span class="page-icon">{{ icon | safe }}</span>{% endif %}
  <h1>{{ page_title }}</h1>
  <article>{{ body_html | safe }}</article>
</main>
{% endblock %}"#;

pub const MINIMAL_INDEX_HTML: &str = r#"{% extends "base.html" %}
{% block content %}
<main>
  <ul>
    {% for entry in entries %}
    <li><a href="{{ entry.url | safe }}">{{ entry.title }}</a></li>
    {% endfor %}
  </ul>
</main>
{% endblock %}"#;

pub const MINIMAL_MACROS_HTML: &str = "";

/// Bundle of UI templates and static assets injected into the headless engine by a shelf variant.
#[derive(Debug, Clone, Copy)]
pub struct ShelfAssets {
    pub base_html: &'static str,
    pub page_html: &'static str,
    pub index_html: &'static str,
    pub macros_html: &'static str,
    pub book_css: &'static str,
    pub book_js: &'static str,
    pub lucide_sprite: &'static str,
    pub simple_sprite: &'static str,
    pub tmt_btn_css: &'static str,
    pub tmt_badge_css: &'static str,
    pub tmt_icon_css: &'static str,
    pub tmt_swatch_css: &'static str,
    pub tmt_switch_css: &'static str,
}

impl ShelfAssets {
    /// Construct a `ShelfAssets` bundle from a variant's templates and compiled CSS/JS,
    /// using the standard Tomet icon sprites and web component stylesheets.
    pub const fn new(
        base_html: &'static str,
        page_html: &'static str,
        index_html: &'static str,
        macros_html: &'static str,
        book_css: &'static str,
        book_js: &'static str,
    ) -> Self {
        Self {
            base_html,
            page_html,
            index_html,
            macros_html,
            book_css,
            book_js,
            lucide_sprite: LUCIDE_SPRITE,
            simple_sprite: SIMPLE_SPRITE,
            tmt_btn_css: TMT_BTN_CSS,
            tmt_badge_css: TMT_BADGE_CSS,
            tmt_icon_css: TMT_ICON_CSS,
            tmt_swatch_css: TMT_SWATCH_CSS,
            tmt_switch_css: TMT_SWITCH_CSS,
        }
    }

    /// Minimal built-in assets for headless testing or barebones rendering.
    pub const fn minimal() -> Self {
        Self::new(
            MINIMAL_BASE_HTML,
            MINIMAL_PAGE_HTML,
            MINIMAL_INDEX_HTML,
            MINIMAL_MACROS_HTML,
            "",
            "",
        )
    }

    pub fn css_version(&self) -> String {
        content_version(self.book_css)
    }

    pub fn js_version(&self) -> String {
        content_version(self.book_js)
    }

    pub fn write_static_assets(
        &self,
        out_dir: &Path,
        src_dir: &Path,
        config: &BookConfig,
    ) -> Result<()> {
        fs::create_dir_all(out_dir)?;

        write_if_changed(&out_dir.join("shelf.css"), self.book_css)
            .context("Failed to write shelf.css")?;
        write_if_changed(&out_dir.join("shelf.js"), self.book_js)
            .context("Failed to write shelf.js")?;

        write_if_changed(&out_dir.join("icons/lucide.svg"), self.lucide_sprite)
            .context("Failed to write icons/lucide.svg")?;
        write_if_changed(&out_dir.join("icons/simple.svg"), self.simple_sprite)
            .context("Failed to write icons/simple.svg")?;

        write_if_changed(&out_dir.join("components/tmt-btn.css"), self.tmt_btn_css)
            .context("Failed to write components/tmt-btn.css")?;
        write_if_changed(
            &out_dir.join("components/tmt-badge.css"),
            self.tmt_badge_css,
        )
        .context("Failed to write components/tmt-badge.css")?;
        write_if_changed(&out_dir.join("components/tmt-icon.css"), self.tmt_icon_css)
            .context("Failed to write components/tmt-icon.css")?;
        write_if_changed(
            &out_dir.join("components/tmt-swatch.css"),
            self.tmt_swatch_css,
        )
        .context("Failed to write components/tmt-swatch.css")?;
        write_if_changed(
            &out_dir.join("components/tmt-switch.css"),
            self.tmt_switch_css,
        )
        .context("Failed to write components/tmt-switch.css")?;

        for css_rel in &config.ui.custom_css {
            let clean_rel = css_rel.trim_start_matches('/');
            let src_file = src_dir.join(clean_rel);
            if src_file.exists() && src_file.is_file() {
                let dest_file = out_dir.join(clean_rel);
                if let Some(parent) = dest_file.parent() {
                    fs::create_dir_all(parent)?;
                }
                let _ = fs::copy(&src_file, &dest_file);
            }
        }

        Ok(())
    }
}

impl Default for ShelfAssets {
    fn default() -> Self {
        Self::minimal()
    }
}

pub fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.exists()
        && let Ok(existing) = fs::read_to_string(path)
        && existing == content
    {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(true)
}
