use anyhow::Result;
use std::path::Path;
use std::sync::LazyLock;

pub use tomet_shelf_config::{BookConfig, i18n};
pub use tomet_shelf_headless::assets::{
    StaticAsset, content_version, write_if_changed, write_static_assets as write_render_assets,
};
pub use tomet_shelf_icons::{LUCIDE_SPRITE, SIMPLE_SPRITE, StandardIconRenderer};
pub mod renderer;
pub use renderer::{BookRenderer, TemplateBundle};

pub const ICON_ASSETS: &[StaticAsset] = &[
    StaticAsset {
        path: "icons/lucide.svg",
        mime: "image/svg+xml",
        content: LUCIDE_SPRITE,
    },
    StaticAsset {
        path: "icons/simple.svg",
        mime: "image/svg+xml",
        content: SIMPLE_SPRITE,
    },
];

// Stylesheets
pub const BOOK_CSS: &str = concat!(
    include_str!("../frontend/styles/00-base.css"),
    include_str!("../frontend/styles/10-shell.css"),
    include_str!("../frontend/styles/20-panes.css"),
    include_str!("../frontend/styles/40-hero.css"),
    include_str!("../frontend/styles/50-nav-infobox.css"),
    include_str!("../frontend/styles/60-content.css"),
    include_str!("../frontend/styles/70-widgets.css"),
    include_str!("../frontend/styles/71-source-viewer.css"),
    include_str!("../frontend/styles/72-center-peek.css"),
    include_str!(concat!(env!("OUT_DIR"), "/tailwind.css")),
    include_str!("../frontend/styles/90-responsive.css"),
    include_str!("../frontend/styles/98-no-js.css"),
);

pub const BOOK_JS: &str = include_str!(concat!(env!("OUT_DIR"), "/shelf.js"));

// Web Component Shadow DOM Stylesheets
pub const TMT_BTN_CSS: &str = include_str!("../frontend/styles/components/tmt-btn.css");
pub const TMT_BADGE_CSS: &str = include_str!("../frontend/styles/components/tmt-badge.css");
pub const TMT_ICON_CSS: &str = include_str!("../frontend/styles/components/tmt-icon.css");
pub const TMT_SWATCH_CSS: &str = include_str!("../frontend/styles/components/tmt-swatch.css");
pub const TMT_SWITCH_CSS: &str = include_str!("../frontend/styles/components/tmt-switch.css");

pub static TMT_BTN_CSS_VERSION: LazyLock<String> = LazyLock::new(|| content_version(TMT_BTN_CSS));
pub static TMT_BADGE_CSS_VERSION: LazyLock<String> =
    LazyLock::new(|| content_version(TMT_BADGE_CSS));
pub static TMT_ICON_CSS_VERSION: LazyLock<String> = LazyLock::new(|| content_version(TMT_ICON_CSS));
pub static TMT_SWATCH_CSS_VERSION: LazyLock<String> =
    LazyLock::new(|| content_version(TMT_SWATCH_CSS));
pub static TMT_SWITCH_CSS_VERSION: LazyLock<String> =
    LazyLock::new(|| content_version(TMT_SWITCH_CSS));

pub const COMPONENT_ASSETS: &[StaticAsset] = &[
    StaticAsset {
        path: "components/tmt-btn.css",
        mime: "text/css; charset=utf-8",
        content: TMT_BTN_CSS,
    },
    StaticAsset {
        path: "components/tmt-badge.css",
        mime: "text/css; charset=utf-8",
        content: TMT_BADGE_CSS,
    },
    StaticAsset {
        path: "components/tmt-icon.css",
        mime: "text/css; charset=utf-8",
        content: TMT_ICON_CSS,
    },
    StaticAsset {
        path: "components/tmt-swatch.css",
        mime: "text/css; charset=utf-8",
        content: TMT_SWATCH_CSS,
    },
    StaticAsset {
        path: "components/tmt-switch.css",
        mime: "text/css; charset=utf-8",
        content: TMT_SWITCH_CSS,
    },
];

pub const COMPONENT_VERSIONS: &[(&str, &str)] = &[
    ("tmt_btn_css_version", TMT_BTN_CSS),
    ("tmt_badge_css_version", TMT_BADGE_CSS),
    ("tmt_icon_css_version", TMT_ICON_CSS),
    ("tmt_swatch_css_version", TMT_SWATCH_CSS),
    ("tmt_switch_css_version", TMT_SWITCH_CSS),
];

// Templates (Single source of truth in tomet-shelf-assets)
pub const BASE_HTML: &str = include_str!("../templates/base.html");
pub const PAGE_HTML: &str = include_str!("../templates/page.html");
pub const INDEX_HTML: &str = include_str!("../templates/index.html");
pub const MACROS_HTML: &str = include_str!("../templates/macros.html");

/// Official `tomet-shelf` UI template and asset bundle.
pub const ASSETS: TemplateBundle = TemplateBundle {
    base_html: BASE_HTML,
    page_html: PAGE_HTML,
    index_html: INDEX_HTML,
    macros_html: MACROS_HTML,
    book_css: BOOK_CSS,
    book_js: BOOK_JS,
    extra_assets: COMPONENT_ASSETS,
    extra_asset_versions: COMPONENT_VERSIONS,
};

pub static CSS_VERSION: LazyLock<String> = LazyLock::new(|| content_version(BOOK_CSS));
pub static JS_VERSION: LazyLock<String> = LazyLock::new(|| content_version(BOOK_JS));

pub fn write_static_assets(out_dir: &Path, src_dir: &Path, config: &BookConfig) -> Result<()> {
    let renderer = BookRenderer::new(config, &ASSETS, false)?;
    use tomet_shelf_headless::PageRenderer;
    write_render_assets(out_dir, src_dir, config, renderer.static_assets())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tomet_shelf_headless::PageRenderer;

    #[test]
    fn assets_versions_are_valid() {
        assert_eq!(CSS_VERSION.len(), 16);
        assert_eq!(JS_VERSION.len(), 16);
        assert_eq!(TMT_BTN_CSS_VERSION.len(), 16);
        assert_eq!(TMT_BADGE_CSS_VERSION.len(), 16);
        assert_eq!(TMT_ICON_CSS_VERSION.len(), 16);
        assert_eq!(TMT_SWATCH_CSS_VERSION.len(), 16);
        assert_eq!(TMT_SWITCH_CSS_VERSION.len(), 16);
    }

    #[test]
    fn tmt_components_are_bundled_in_book_assets() {
        assert!(
            BOOK_JS.contains("customElements.define('tmt-btn'")
                || BOOK_JS.contains("customElements.define(\"tmt-btn\""),
            "tmt-btn custom element is defined in BOOK_JS"
        );
        assert!(
            BOOK_JS.contains("customElements.define('tmt-badge'")
                || BOOK_JS.contains("customElements.define(\"tmt-badge\""),
            "tmt-badge custom element is defined in BOOK_JS"
        );
        assert!(
            BOOK_JS.contains("customElements.define('tmt-icon'")
                || BOOK_JS.contains("customElements.define(\"tmt-icon\""),
            "tmt-icon custom element is defined in BOOK_JS"
        );
        assert!(
            BOOK_CSS.contains("tmt-btn"),
            "tmt-btn light DOM base rules are in BOOK_CSS"
        );
    }

    #[test]
    fn template_bundle_renders_base_path_correctly() {
        let mut config = BookConfig::default();
        config.book.title = "Docs".into();
        config.build.base_path = Some("/sub".into());
        let renderer = BookRenderer::new(&config, &ASSETS, false).unwrap();
        let index_html = PageRenderer::render_index(&renderer, &[], &[]).unwrap();
        assert!(index_html.contains("href=\"/sub/shelf.css?v="));
        assert!(
            index_html.contains("href=\"/sub/components/tmt-btn.css?v="),
            "tmt-btn shadow stylesheet link inside macro uses base_path"
        );
        let paths: Vec<&str> = renderer.static_assets().iter().map(|a| a.path).collect();
        assert!(paths.contains(&"icons/lucide.svg"));
        assert!(paths.contains(&"icons/simple.svg"));
    }
}
