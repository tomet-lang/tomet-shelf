use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::assets::StaticAsset;
use crate::document::ProcessedDoc;
use tomet_shelf_config::BookConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSummary {
    pub name: String,
    pub count: usize,
}

/// A page that points at the one being rendered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backlink {
    pub url: String,
    pub title: String,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrySummary {
    pub url: String,
    pub title: String,
    pub section: Option<String>,
    /// Pages the index nested under this one. Always empty when the catalog
    /// is generated rather than written.
    #[serde(default)]
    pub children: Vec<EntrySummary>,
}

/// Template-engine-agnostic page renderer implemented by the application/theme layer.
pub trait PageRenderer: Send + Sync {
    /// Render a single document page into a complete HTML string.
    fn render_page(
        &self,
        doc: &ProcessedDoc,
        backlinks: &[Backlink],
        book_index: &[EntrySummary],
    ) -> Result<String>;

    /// Render the catalog/index page into a complete HTML string.
    fn render_index(&self, sections: &[SectionSummary], entries: &[EntrySummary])
    -> Result<String>;

    /// Static assets (CSS, JS, etc.) supplied by this theme.
    fn static_assets(&self) -> &[StaticAsset] {
        &[]
    }
}

/// Factory that creates a [`PageRenderer`] bound to a [`BookConfig`].
/// Used by `build_book` and by the dev server (which re-creates the renderer on config hot-reload).
pub trait RendererFactory: Send + Sync {
    fn create(&self, config: &BookConfig, is_dev: bool) -> Result<Box<dyn PageRenderer>>;
}

impl<F> RendererFactory for F
where
    F: Fn(&BookConfig, bool) -> Result<Box<dyn PageRenderer>> + Send + Sync,
{
    fn create(&self, config: &BookConfig, is_dev: bool) -> Result<Box<dyn PageRenderer>> {
        self(config, is_dev)
    }
}

const MINIMAL_STATIC_ASSETS: &[StaticAsset] = &[
    StaticAsset {
        path: "shelf.css",
        mime: "text/css; charset=utf-8",
        content: "",
    },
    StaticAsset {
        path: "shelf.js",
        mime: "application/javascript; charset=utf-8",
        content: "",
    },
];

/// Minimal template-engine-free renderer for headless tests and barebones rendering.
#[derive(Debug, Clone, Default)]
pub struct MinimalRenderer {
    site_title: String,
    lang: String,
    default_view: String,
    base_path: String,
    clean_url_prefix: String,
    full_url_prefix: String,
    home_url: String,
    strings_json: String,
}

impl MinimalRenderer {
    pub fn new(config: &BookConfig) -> Self {
        let base_path = config.build.clean_base_path().to_string();
        let clean_url_prefix = config.build.clean_url_prefix().to_string();
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
        let strings_json =
            serde_json::to_string(&config.ui.strings).unwrap_or_else(|_| "{}".to_string());

        Self {
            site_title: config.book.title.clone(),
            lang: config.book.lang.clone(),
            default_view: config.ui.default_view.clone(),
            base_path,
            clean_url_prefix,
            full_url_prefix,
            home_url,
            strings_json,
        }
    }

    fn wrap_base(&self, page_title: Option<&str>, content_html: &str) -> String {
        let title_prefix = match page_title {
            Some(t) if !t.is_empty() => format!("{t} | "),
            _ => String::new(),
        };
        let base = &self.base_path;
        let site_title = &self.site_title;
        let lang = &self.lang;
        let view = &self.default_view;
        let home_url = &self.home_url;
        let base_json = serde_json::to_string(&self.base_path).unwrap_or_default();
        let clean_prefix_json = serde_json::to_string(&self.clean_url_prefix).unwrap_or_default();
        let full_prefix_json = serde_json::to_string(&self.full_url_prefix).unwrap_or_default();
        let home_url_json = serde_json::to_string(&self.home_url).unwrap_or_default();
        let strings_json = &self.strings_json;

        format!(
            r#"<!DOCTYPE html>
<html lang="{lang}" data-view="{view}">
<head>
  <meta charset="utf-8" />
  <title>{title_prefix}{site_title}</title>
  <link rel="stylesheet" href="{base}/shelf.css" />
</head>
<body>
  <a href="{home_url}">{site_title}</a>
  {content_html}
  <script>
    window.tmtBasePath = {base_json};
    window.tmtUrlPrefix = {clean_prefix_json};
    window.tmtFullUrlPrefix = {full_prefix_json};
    window.tmtHomeUrl = {home_url_json};
    window.tmtStrings = {strings_json};
  </script>
  <script src="{base}/shelf.js"></script>
</body>
</html>"#
        )
    }
}

impl PageRenderer for MinimalRenderer {
    fn render_page(
        &self,
        doc: &ProcessedDoc,
        _backlinks: &[Backlink],
        _book_index: &[EntrySummary],
    ) -> Result<String> {
        let slug_span = if !doc.slug.is_empty() {
            format!(
                r#"<span class="sr-only" data-pagefind-meta="path" data-pagefind-weight="10.0">{}</span>"#,
                doc.slug
            )
        } else {
            String::new()
        };
        let rel_span = if !doc.rel_path.is_empty() && doc.rel_path != doc.slug {
            format!(
                r#"<span class="sr-only" data-pagefind-weight="10.0">{}</span>"#,
                doc.rel_path
            )
        } else {
            String::new()
        };
        let icon_span = match &doc.icon {
            Some(ic) => format!(r#"<span class="page-icon">{ic}</span>"#),
            None => String::new(),
        };
        let content = format!(
            r#"<main data-pagefind-body>
  {slug_span}
  {rel_span}
  {icon_span}
  <h1>{}</h1>
  <article>{}</article>
</main>"#,
            doc.title, doc.body_html
        );
        Ok(self.wrap_base(Some(&doc.title), &content))
    }

    fn render_index(
        &self,
        _sections: &[SectionSummary],
        entries: &[EntrySummary],
    ) -> Result<String> {
        let mut items = String::new();
        for entry in entries {
            items.push_str(&format!(
                r#"<li><a href="{}">{}</a></li>"#,
                entry.url, entry.title
            ));
        }
        let content = format!("<main><ul>{items}</ul></main>");
        Ok(self.wrap_base(None, &content))
    }

    fn static_assets(&self) -> &[StaticAsset] {
        MINIMAL_STATIC_ASSETS
    }
}

/// Unit factory that creates a [`MinimalRenderer`] for headless tests.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinimalRendererFactory;

impl RendererFactory for MinimalRendererFactory {
    fn create(&self, config: &BookConfig, _is_dev: bool) -> Result<Box<dyn PageRenderer>> {
        Ok(Box::new(MinimalRenderer::new(config)))
    }
}
