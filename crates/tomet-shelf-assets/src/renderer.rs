//| Askama page template renderer for `tomet-shelf`.

use anyhow::Result;
use askama::Template;
use std::collections::HashMap;
use tomet_shelf_config::BookConfig;
use tomet_shelf_document::{HeroChipItem, InfoboxRowItem, ProcessedDoc, SectionTab, TocItem};
use tomet_shelf_headless::{
    Backlink, EntrySummary, PageRenderer, RendererFactory, SectionSummary, StaticAsset,
    content_version,
};

//==============================================================================
// HTML Helper Functions
//==============================================================================

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

fn render_book_index_items(buf: &mut String, items: &[EntrySummary], here: &str) {
    buf.push_str("<ul class=\"book-index-items list-none m-0 p-0 flex flex-col gap-0.5\">");
    for entry in items {
        buf.push_str("<li class=\"book-index-item\">");
        let is_here = entry.url == here;
        let here_classes = if is_here {
            " is-here font-semibold text-accent-primary bg-subtle"
        } else {
            ""
        };
        let aria_current = if is_here {
            " aria-current=\"page\""
        } else {
            ""
        };
        buf.push_str(&format!(
            "<a href=\"{}\" class=\"book-index-link flex items-center justify-between py-1.5 px-2.5 text-[0.825rem] text-text-secondary hover:text-text hover:bg-subtle rounded-md transition-colors no-underline{}\"{}>\n      <span class=\"book-index-title overflow-hidden text-ellipsis whitespace-nowrap\">{}</span>\n    </a>",
            html_escape(&entry.url),
            here_classes,
            aria_current,
            html_escape(&entry.title)
        ));
        if !entry.children.is_empty() {
            buf.push_str("<div class=\"book-index-children ml-3 pl-2 border-l border-border-subtle my-0.5 flex flex-col gap-0.5\">");
            render_book_index_items(buf, &entry.children, here);
            buf.push_str("</div>");
        }
        buf.push_str("</li>");
    }
    buf.push_str("</ul>");
}

fn render_catalog_entries(buf: &mut String, items: &[EntrySummary], top: bool) {
    if top {
        buf.push_str("<ul class=\"book-index-items list-none m-0 p-0 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3\">");
    } else {
        buf.push_str("<ul class=\"book-index-items list-none m-0 p-0 flex flex-col gap-1.5 ml-3 mt-1.5 pl-2.5 border-l border-border-subtle\">");
    }
    for entry in items {
        buf.push_str("<li class=\"book-index-item\">");
        buf.push_str(&format!(
            "<a href=\"{}\" class=\"book-index-link block p-3.5 rounded-xl bg-surface shadow-sm hover:shadow-md transition-all no-underline text-text\">\n      <span class=\"book-index-title font-semibold text-sm block overflow-hidden text-ellipsis whitespace-nowrap\">{}</span>",
            html_escape(&entry.url),
            html_escape(&entry.title)
        ));
        if let Some(ref section) = entry.section {
            buf.push_str(&format!(
                "\n      <span class=\"book-index-meta text-xs text-text-muted block mt-1\">{}</span>",
                html_escape(section)
            ));
        }
        buf.push_str("\n    </a>");
        if !entry.children.is_empty() {
            render_catalog_entries(buf, &entry.children, false);
        }
        buf.push_str("</li>");
    }
    buf.push_str("</ul>");
}

//==============================================================================
// Askama Templates
//==============================================================================

#[derive(Template)]
#[template(path = "page.html")]
pub struct PageTemplate<'a> {
    pub site_title: &'a str,
    pub description: Option<&'a str>,
    pub lang: &'a str,
    pub default_view: &'a str,
    pub custom_css: &'a [String],
    pub css_version: &'a str,
    pub js_version: &'a str,
    pub base_path: &'a str,
    pub clean_url_prefix: &'a str,
    pub full_url_prefix: &'a str,
    pub home_url: &'a str,
    pub strings: &'a HashMap<String, String>,

    pub tmt_btn_css_version: &'a str,
    pub tmt_badge_css_version: &'a str,
    pub tmt_icon_css_version: &'a str,
    pub tmt_swatch_css_version: &'a str,
    pub tmt_switch_css_version: &'a str,

    pub source_path: &'a str,
    pub rel_path: &'a str,
    pub slug: &'a str,
    pub page_title: &'a str,
    pub section: Option<&'a str>,
    pub kind: Option<&'a str>,
    pub primary_color: Option<&'a str>,
    pub icon: Option<&'a str>,
    pub icon_image_url: Option<&'a str>,
    pub banner_url: Option<&'a str>,
    pub banner_original_url: Option<&'a str>,
    pub banner_y: Option<f64>,
    pub images: &'a [String],
    pub original_images: &'a [String],
    pub hero_chips: &'a [HeroChipItem],
    pub infobox_rows: &'a [InfoboxRowItem],
    pub has_data: bool,
    pub toc: &'a [TocItem],
    pub section_tabs: &'a [SectionTab],
    pub body_html: &'a str,
    pub backlinks: &'a [Backlink],
    pub book_index: &'a [EntrySummary],
    pub page_url: &'a str,
    pub rail_letters: &'a [String],
    pub is_dev: bool,
}

impl<'a> PageTemplate<'a> {
    pub fn t(&self, key: &str, default: &'a str) -> &'a str {
        if let Some(val) = self.strings.get(key) {
            val.as_str()
        } else {
            default
        }
    }

    pub fn base_path_json(&self) -> String {
        serde_json::to_string(&self.base_path).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn clean_url_prefix_json(&self) -> String {
        serde_json::to_string(&self.clean_url_prefix).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn full_url_prefix_json(&self) -> String {
        serde_json::to_string(&self.full_url_prefix).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn home_url_json(&self) -> String {
        serde_json::to_string(&self.home_url).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn strings_json(&self) -> String {
        serde_json::to_string(&self.strings).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn page_url_json(&self) -> String {
        serde_json::to_string(&self.page_url).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn book_index_html(&self) -> String {
        let mut buf = String::new();
        render_book_index_items(&mut buf, self.book_index, self.page_url);
        buf
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub site_title: &'a str,
    pub description: Option<&'a str>,
    pub lang: &'a str,
    pub default_view: &'a str,
    pub custom_css: &'a [String],
    pub css_version: &'a str,
    pub js_version: &'a str,
    pub base_path: &'a str,
    pub clean_url_prefix: &'a str,
    pub full_url_prefix: &'a str,
    pub home_url: &'a str,
    pub strings: &'a HashMap<String, String>,

    pub tmt_btn_css_version: &'a str,
    pub tmt_badge_css_version: &'a str,
    pub tmt_icon_css_version: &'a str,
    pub tmt_swatch_css_version: &'a str,
    pub tmt_switch_css_version: &'a str,

    pub page_title: &'a str,

    pub sections: &'a [SectionSummary],
    pub entries: &'a [EntrySummary],
}

impl<'a> IndexTemplate<'a> {
    pub fn t(&self, key: &str, default: &'a str) -> &'a str {
        if let Some(val) = self.strings.get(key) {
            val.as_str()
        } else {
            default
        }
    }

    pub fn base_path_json(&self) -> String {
        serde_json::to_string(&self.base_path).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn clean_url_prefix_json(&self) -> String {
        serde_json::to_string(&self.clean_url_prefix).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn full_url_prefix_json(&self) -> String {
        serde_json::to_string(&self.full_url_prefix).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn home_url_json(&self) -> String {
        serde_json::to_string(&self.home_url).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn strings_json(&self) -> String {
        serde_json::to_string(&self.strings).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn entries_html(&self) -> String {
        let mut buf = String::new();
        render_catalog_entries(&mut buf, self.entries, true);
        buf
    }
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate<'a> {
    pub site_title: &'a str,
    pub description: Option<&'a str>,
    pub lang: &'a str,
    pub custom_css: &'a [String],
    pub css_version: &'a str,
    pub js_version: &'a str,
    pub base_path: &'a str,
    pub clean_url_prefix: &'a str,
    pub full_url_prefix: &'a str,
    pub home_url: &'a str,
    pub strings: &'a HashMap<String, String>,

    pub tmt_btn_css_version: &'a str,
    pub tmt_badge_css_version: &'a str,
    pub tmt_icon_css_version: &'a str,
    pub tmt_swatch_css_version: &'a str,
    pub tmt_switch_css_version: &'a str,

    pub page_title: &'a str,
    pub missing_path: &'a str,
}

impl<'a> NotFoundTemplate<'a> {
    pub fn t(&self, key: &str, default: &'a str) -> &'a str {
        if let Some(val) = self.strings.get(key) {
            val.as_str()
        } else {
            default
        }
    }

    pub fn base_path_json(&self) -> String {
        serde_json::to_string(&self.base_path).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn clean_url_prefix_json(&self) -> String {
        serde_json::to_string(&self.clean_url_prefix).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn full_url_prefix_json(&self) -> String {
        serde_json::to_string(&self.full_url_prefix).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn home_url_json(&self) -> String {
        serde_json::to_string(&self.home_url).unwrap_or_else(|_| "\"\"".to_string())
    }

    pub fn strings_json(&self) -> String {
        serde_json::to_string(&self.strings).unwrap_or_else(|_| "{}".to_string())
    }
}

//==============================================================================
// TemplateBundle
//==============================================================================

/// Static HTML templates, CSS/JS assets, and extra static files for a theme.
#[derive(Debug, Clone, Copy)]
pub struct TemplateBundle {
    pub base_html: &'static str,
    pub page_html: &'static str,
    pub index_html: &'static str,
    pub not_found_html: &'static str,
    pub macros_html: &'static str,
    pub book_css: &'static str,
    pub book_js: &'static str,
    /// Additional static assets (e.g. Web Component Shadow DOM stylesheets).
    pub extra_assets: &'static [StaticAsset],
    /// Additional template globals `(global_name, asset_content_to_hash)`.
    pub extra_asset_versions: &'static [(&'static str, &'static str)],
}

impl RendererFactory for TemplateBundle {
    fn create(&self, config: &BookConfig, is_dev: bool) -> Result<Box<dyn PageRenderer>> {
        Ok(Box::new(BookRenderer::new(config, self, is_dev)?))
    }

    fn icon_renderer(&self) -> Option<std::sync::Arc<dyn tomet_shelf_icons::IconRenderer>> {
        Some(std::sync::Arc::new(
            tomet_shelf_icons::StandardIconRenderer::new(),
        ))
    }
}

//==============================================================================
// BookRenderer
//==============================================================================

/// Holds Askama templates configuration and static assets for rendering book pages.
pub struct BookRenderer {
    config: BookConfig,
    clean_url_prefix: String,
    full_url_prefix: String,
    home_url: String,
    base_path: String,
    css_version: String,
    js_version: String,
    tmt_btn_css_version: String,
    tmt_badge_css_version: String,
    tmt_icon_css_version: String,
    tmt_swatch_css_version: String,
    tmt_switch_css_version: String,
    rail_letters: Vec<String>,
    static_assets: Vec<StaticAsset>,
    link_icons: std::collections::HashMap<String, String>,
    is_dev: bool,
}

impl BookRenderer {
    pub fn new(config: &BookConfig, bundle: &TemplateBundle, is_dev: bool) -> Result<Self> {
        let rail_title = config
            .ui
            .rail_title
            .as_deref()
            .unwrap_or(&config.book.title);

        let rail_letters: Vec<String> = rail_title.chars().map(|c| c.to_string()).collect();

        let base_path = config.build.clean_base_path().to_string();
        let full_url_prefix = config.build.full_url_prefix();
        let home_url = if full_url_prefix.is_empty() {
            if base_path.is_empty() {
                "/".to_string()
            } else {
                format!("{base_path}/")
            }
        } else {
            full_url_prefix.clone()
        };

        let mut tmt_btn_css_version = String::new();
        let mut tmt_badge_css_version = String::new();
        let mut tmt_icon_css_version = String::new();
        let mut tmt_swatch_css_version = String::new();
        let mut tmt_switch_css_version = String::new();

        for &(var_name, content) in bundle.extra_asset_versions {
            let ver = content_version(content);
            match var_name {
                "tmt_btn_css_version" => tmt_btn_css_version = ver,
                "tmt_badge_css_version" => tmt_badge_css_version = ver,
                "tmt_icon_css_version" => tmt_icon_css_version = ver,
                "tmt_swatch_css_version" => tmt_swatch_css_version = ver,
                "tmt_switch_css_version" => tmt_switch_css_version = ver,
                _ => {}
            }
        }

        let mut static_assets =
            Vec::with_capacity(2 + crate::ICON_ASSETS.len() + bundle.extra_assets.len());
        static_assets.push(StaticAsset {
            path: "shelf.css",
            mime: "text/css; charset=utf-8",
            content: bundle.book_css,
        });
        static_assets.push(StaticAsset {
            path: "shelf.js",
            mime: "application/javascript; charset=utf-8",
            content: bundle.book_js,
        });
        static_assets.extend_from_slice(crate::ICON_ASSETS);
        for asset in bundle.extra_assets {
            if !static_assets.iter().any(|a| a.path == asset.path) {
                static_assets.push(*asset);
            }
        }

        let mut config = config.clone();
        crate::i18n::apply_ui_defaults(&mut config.ui.strings, &config.book.lang);

        Ok(Self {
            config,
            clean_url_prefix: full_url_prefix.clone(),
            full_url_prefix,
            home_url,
            base_path,
            css_version: content_version(bundle.book_css),
            js_version: content_version(bundle.book_js),
            tmt_btn_css_version,
            tmt_badge_css_version,
            tmt_icon_css_version,
            tmt_swatch_css_version,
            tmt_switch_css_version,
            rail_letters,
            static_assets,
            link_icons: std::collections::HashMap::new(),
            is_dev,
        })
    }

    pub fn render_page(
        &self,
        doc: &ProcessedDoc,
        backlinks: &[Backlink],
        book_index: &[EntrySummary],
    ) -> Result<String> {
        let page_url = format!("{}/{}", self.clean_url_prefix, doc.slug);

        let icon_renderer = tomet_shelf_icons::StandardIconRenderer::new();
        let rendered_icon = doc.icon.as_ref().map(|ic| {
            tomet_shelf_render::render_page_icon(ic, &self.base_path, Some(&icon_renderer))
        });

        let body_html = tomet_shelf_render::enhance_body_html(
            &doc.body_html,
            &self.config,
            &self.base_path,
            Some(&icon_renderer),
        );
        let body_html = if self.config.ui.links.note_icons && !self.link_icons.is_empty() {
            tomet_shelf_render::enhance_internal_links(
                &body_html,
                &self.link_icons,
                &self.full_url_prefix,
            )
        } else {
            body_html
        };

        let template = PageTemplate {
            site_title: &self.config.book.title,
            description: self.config.book.description.as_deref(),
            lang: &self.config.book.lang,
            default_view: &self.config.ui.default_view,
            custom_css: &self.config.ui.custom_css,
            css_version: &self.css_version,
            js_version: &self.js_version,
            base_path: &self.base_path,
            clean_url_prefix: self.config.build.clean_url_prefix(),
            full_url_prefix: &self.full_url_prefix,
            home_url: &self.home_url,
            strings: &self.config.ui.strings,

            tmt_btn_css_version: &self.tmt_btn_css_version,
            tmt_badge_css_version: &self.tmt_badge_css_version,
            tmt_icon_css_version: &self.tmt_icon_css_version,
            tmt_swatch_css_version: &self.tmt_swatch_css_version,
            tmt_switch_css_version: &self.tmt_switch_css_version,

            source_path: &doc.source_path,
            rel_path: &doc.rel_path,
            slug: &doc.slug,
            page_title: &doc.title,
            section: doc.section.as_deref(),
            kind: doc.kind.as_deref(),
            primary_color: doc.primary_color.as_deref(),
            icon: rendered_icon.as_deref(),
            icon_image_url: doc.icon_image_url.as_deref(),
            banner_url: doc.banner_url.as_deref(),
            banner_original_url: doc.banner_original_url.as_deref(),
            banner_y: doc.banner_y,
            images: &doc.images,
            original_images: &doc.original_images,
            hero_chips: &doc.hero_chips,
            infobox_rows: &doc.infobox_rows,
            has_data: doc.has_data,
            toc: &doc.toc,
            section_tabs: &doc.section_tabs,
            body_html: &body_html,
            backlinks,
            book_index,
            page_url: &page_url,
            rail_letters: &self.rail_letters,
            is_dev: self.is_dev,
        };

        template.render().map_err(Into::into)
    }

    pub fn render_index(
        &self,
        sections: &[SectionSummary],
        entries: &[EntrySummary],
    ) -> Result<String> {
        let template = IndexTemplate {
            site_title: &self.config.book.title,
            description: self.config.book.description.as_deref(),
            lang: &self.config.book.lang,
            default_view: &self.config.ui.default_view,
            custom_css: &self.config.ui.custom_css,
            css_version: &self.css_version,
            js_version: &self.js_version,
            base_path: &self.base_path,
            clean_url_prefix: self.config.build.clean_url_prefix(),
            full_url_prefix: &self.full_url_prefix,
            home_url: &self.home_url,
            strings: &self.config.ui.strings,

            tmt_btn_css_version: &self.tmt_btn_css_version,
            tmt_badge_css_version: &self.tmt_badge_css_version,
            tmt_icon_css_version: &self.tmt_icon_css_version,
            tmt_swatch_css_version: &self.tmt_swatch_css_version,
            tmt_switch_css_version: &self.tmt_switch_css_version,

            page_title: "",
            sections,
            entries,
        };

        template.render().map_err(Into::into)
    }

    pub fn render_not_found(&self, missing_path: &str) -> Result<String> {
        let template = NotFoundTemplate {
            site_title: &self.config.book.title,
            description: self.config.book.description.as_deref(),
            lang: &self.config.book.lang,
            custom_css: &self.config.ui.custom_css,
            css_version: &self.css_version,
            js_version: &self.js_version,
            base_path: &self.base_path,
            clean_url_prefix: self.config.build.clean_url_prefix(),
            full_url_prefix: &self.full_url_prefix,
            home_url: &self.home_url,
            strings: &self.config.ui.strings,

            tmt_btn_css_version: &self.tmt_btn_css_version,
            tmt_badge_css_version: &self.tmt_badge_css_version,
            tmt_icon_css_version: &self.tmt_icon_css_version,
            tmt_swatch_css_version: &self.tmt_swatch_css_version,
            tmt_switch_css_version: &self.tmt_switch_css_version,

            page_title: "404 Not Found",
            missing_path,
        };

        template.render().map_err(Into::into)
    }
}

impl PageRenderer for BookRenderer {
    fn render_page(
        &self,
        doc: &ProcessedDoc,
        backlinks: &[Backlink],
        book_index: &[EntrySummary],
    ) -> Result<String> {
        BookRenderer::render_page(self, doc, backlinks, book_index)
    }

    fn render_index(
        &self,
        sections: &[SectionSummary],
        entries: &[EntrySummary],
    ) -> Result<String> {
        BookRenderer::render_index(self, sections, entries)
    }

    fn render_not_found(&self, missing_path: &str) -> Result<String> {
        BookRenderer::render_not_found(self, missing_path)
    }

    fn static_assets(&self) -> &[StaticAsset] {
        &self.static_assets
    }

    fn icon_renderer(&self) -> Option<std::sync::Arc<dyn tomet_shelf_icons::IconRenderer>> {
        Some(std::sync::Arc::new(
            tomet_shelf_icons::StandardIconRenderer::new(),
        ))
    }

    fn set_doc_icons(
        &mut self,
        icons: &std::collections::HashMap<String, tomet_shelf_document::DocIcon>,
    ) {
        let icon_renderer = tomet_shelf_icons::StandardIconRenderer::new();
        self.link_icons = icons
            .iter()
            .map(|(slug, icon)| {
                let html = tomet_shelf_render::render_page_link_icon(
                    icon,
                    &self.base_path,
                    Some(&icon_renderer),
                );
                (slug.clone(), html)
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn book_renderer_applies_japanese_ui_strings_by_default() {
        let config = BookConfig::default();
        let renderer = BookRenderer::new(&config, &crate::ASSETS, false).unwrap();
        assert_eq!(
            renderer
                .config
                .ui
                .strings
                .get("pane.toc")
                .map(String::as_str),
            Some("目次")
        );
        assert_eq!(
            renderer
                .config
                .ui
                .strings
                .get("search.placeholder")
                .map(String::as_str),
            Some("検索... (Ctrl+K)")
        );
    }

    #[test]
    fn book_renderer_applies_english_ui_strings_for_english_lang() {
        let mut config = BookConfig::default();
        config.book.lang = "en".to_string();
        let renderer = BookRenderer::new(&config, &crate::ASSETS, false).unwrap();
        assert_eq!(
            renderer
                .config
                .ui
                .strings
                .get("pane.toc")
                .map(String::as_str),
            Some("Table of Contents")
        );
        assert_eq!(
            renderer
                .config
                .ui
                .strings
                .get("search.placeholder")
                .map(String::as_str),
            Some("Search... (Ctrl+K)")
        );
    }

    #[test]
    fn book_renderer_preserves_user_overrides() {
        let mut config = BookConfig::default();
        config
            .ui
            .strings
            .insert("pane.toc".to_string(), "Custom Outline".to_string());
        let renderer = BookRenderer::new(&config, &crate::ASSETS, false).unwrap();
        assert_eq!(
            renderer
                .config
                .ui
                .strings
                .get("pane.toc")
                .map(String::as_str),
            Some("Custom Outline")
        );
        assert_eq!(
            renderer
                .config
                .ui
                .strings
                .get("pane.links")
                .map(String::as_str),
            Some("リンク")
        );
    }
}
