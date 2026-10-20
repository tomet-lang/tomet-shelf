use anyhow::Result;
use std::path::Path;
use std::sync::LazyLock;

pub use tomet_shelf_config::{BookConfig, i18n};
pub use tomet_shelf_render::assets::{
    LUCIDE_SPRITE, SIMPLE_SPRITE, ShelfAssets, TMT_BADGE_CSS, TMT_BADGE_CSS_VERSION, TMT_BTN_CSS,
    TMT_BTN_CSS_VERSION, TMT_ICON_CSS, TMT_ICON_CSS_VERSION, TMT_SWATCH_CSS,
    TMT_SWATCH_CSS_VERSION, TMT_SWITCH_CSS, TMT_SWITCH_CSS_VERSION, content_version,
    write_if_changed,
};

// Stylesheets
pub const BOOK_CSS: &str = concat!(
    include_str!("../frontend/styles/00-base.css"),
    include_str!("../frontend/styles/10-shell.css"),
    include_str!("../frontend/styles/20-panes.css"),
    include_str!("../frontend/styles/30-sticky-tabs.css"),
    include_str!("../frontend/styles/35-sticky-tabs-vertical.css"),
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

// Templates
pub const BASE_HTML: &str = include_str!("../frontend/templates/base.html");
pub const PAGE_HTML: &str = include_str!("../frontend/templates/page.html");
pub const INDEX_HTML: &str = include_str!("../frontend/templates/index.html");
pub const MACROS_HTML: &str = include_str!("../frontend/templates/macros.html");

/// Official `tomet-shelf` UI asset bundle.
pub const ASSETS: ShelfAssets = ShelfAssets::new(
    BASE_HTML,
    PAGE_HTML,
    INDEX_HTML,
    MACROS_HTML,
    BOOK_CSS,
    BOOK_JS,
);

pub static CSS_VERSION: LazyLock<String> = LazyLock::new(|| content_version(BOOK_CSS));
pub static JS_VERSION: LazyLock<String> = LazyLock::new(|| content_version(BOOK_JS));

pub fn write_static_assets(out_dir: &Path, src_dir: &Path, config: &BookConfig) -> Result<()> {
    ASSETS.write_static_assets(out_dir, src_dir, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_versions_are_valid() {
        assert_eq!(CSS_VERSION.len(), 16);
        assert_eq!(JS_VERSION.len(), 16);
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
}
