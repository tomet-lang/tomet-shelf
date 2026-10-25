//| MiniJinja page template renderer for `tomet-shelf`.

use anyhow::{Context, Result};
use minijinja::{Environment, Value, context};
use tomet_shelf_config::BookConfig;
use tomet_shelf_render::{
    Backlink, EntrySummary, PageRenderer, ProcessedDoc, RendererFactory, SectionSummary,
    StaticAsset, content_version,
};

//==============================================================================
// TemplateBundle
//==============================================================================

/// Static HTML templates, CSS/JS assets, and extra static files for a theme.
#[derive(Debug, Clone, Copy)]
pub struct TemplateBundle {
    pub base_html: &'static str,
    pub page_html: &'static str,
    pub index_html: &'static str,
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
}

//==============================================================================
// BookRenderer
//==============================================================================

/// Holds compiled MiniJinja templates and static assets for rendering book pages.
pub struct BookRenderer {
    env: Environment<'static>,
    clean_url_prefix: String,
    static_assets: Vec<StaticAsset>,
}

impl BookRenderer {
    pub fn new(config: &BookConfig, bundle: &TemplateBundle, is_dev: bool) -> Result<Self> {
        let mut env = Environment::new();

        env.add_template("base.html", bundle.base_html)
            .context("Failed to add base.html template")?;
        env.add_template("page.html", bundle.page_html)
            .context("Failed to add page.html template")?;
        env.add_template("index.html", bundle.index_html)
            .context("Failed to add index.html template")?;
        env.add_template("macros.html", bundle.macros_html)
            .context("Failed to add macros.html template")?;

        let rail_title = config
            .ui
            .rail_title
            .as_deref()
            .unwrap_or(&config.book.title);

        let rail_letters: Vec<String> = rail_title.chars().map(|c| c.to_string()).collect();

        // Vault-wide invariant values registered once as globals rather than
        // rebuilt per-page in context!().
        env.add_global("site_title", Value::from(&config.book.title));
        env.add_global(
            "description",
            Value::from_serialize(&config.book.description),
        );
        env.add_global("lang", Value::from(&config.book.lang));
        env.add_global("default_view", Value::from(&config.ui.default_view));
        env.add_global("custom_css", Value::from_serialize(&config.ui.custom_css));
        env.add_global("t", Value::from_serialize(&config.ui.strings));
        env.add_global("css_version", Value::from(content_version(bundle.book_css)));
        env.add_global("js_version", Value::from(content_version(bundle.book_js)));
        for &(var_name, asset_content) in bundle.extra_asset_versions {
            env.add_global(var_name, Value::from(content_version(asset_content)));
        }
        env.add_global("is_dev", Value::from(is_dev));
        env.add_global("rail_letters", Value::from_serialize(&rail_letters));

        let base_path = config.build.clean_base_path();
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

        env.add_global("base_path", Value::from(base_path));
        env.add_global(
            "clean_url_prefix",
            Value::from(config.build.clean_url_prefix()),
        );
        env.add_global("full_url_prefix", Value::from(&full_url_prefix));
        env.add_global("home_url", Value::from(&home_url));

        let mut static_assets = Vec::with_capacity(2 + bundle.extra_assets.len());
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
        static_assets.extend_from_slice(bundle.extra_assets);

        Ok(Self {
            env,
            clean_url_prefix: full_url_prefix,
            static_assets,
        })
    }

    pub fn render_page(
        &self,
        doc: &ProcessedDoc,
        backlinks: &[Backlink],
        book_index: &[EntrySummary],
    ) -> Result<String> {
        let tmpl = self.env.get_template("page.html")?;

        let ctx = context! {
            source_path => &doc.source_path,
            rel_path => &doc.rel_path,
            slug => &doc.slug,
            page_title => &doc.title,
            section => &doc.section,
            kind => &doc.kind,
            primary_color => &doc.primary_color,
            icon => &doc.icon,
            icon_image_url => &doc.icon_image_url,
            banner_url => &doc.banner_url,
            banner_original_url => &doc.banner_original_url,
            banner_y => doc.banner_y,
            images => &doc.images,
            original_images => &doc.original_images,
            hero_chips => &doc.hero_chips,
            infobox_rows => &doc.infobox_rows,
            has_data => doc.has_data,
            toc => &doc.toc,
            section_tabs => &doc.section_tabs,
            body_html => &doc.body_html,
            backlinks => backlinks,
            book_index => book_index,
            // So the index can mark where the reader is standing.
            page_url => format!("{}/{}", self.clean_url_prefix, doc.slug),
        };

        let rendered = tmpl.render(ctx)?;
        Ok(rendered)
    }

    pub fn render_index(
        &self,
        sections: &[SectionSummary],
        entries: &[EntrySummary],
    ) -> Result<String> {
        let tmpl = self.env.get_template("index.html")?;

        let ctx = context! {
            sections => sections,
            entries => entries,
        };

        let rendered = tmpl.render(ctx)?;
        Ok(rendered)
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

    fn static_assets(&self) -> &[StaticAsset] {
        &self.static_assets
    }
}
