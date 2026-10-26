//! HTML tag injection and presentation rendering for `tomet-shelf`.
//!
//! Provides post-processing and HTML tag injection for document markup:
//! - Code block decoration (language badge, copy button)
//! - List marker enhancement (task checkboxes, badge pills)
//! - Link icon enhancement (external brand/favicons, internal note icons)
//! - `@doc.icon` custom element rendering
//! - HTML escaping and presentation formatters

pub mod code;
pub mod doc_icon;
pub mod external_links;
pub mod html;
pub mod internal_links;
pub mod marker;
pub mod meta;

pub use code::enhance_code_blocks;
pub use doc_icon::{create_doc_icon_custom_element, render_doc_icon};
pub use external_links::enhance_external_links;
pub use html::{
    color_chip_html, escape_html, external_link_html, is_external_url, sanitize_color_hex,
};
pub use internal_links::enhance_internal_links;
pub use marker::enhance_list_markers;
pub use meta::{format_text_icon, format_text_icon_link, render_page_icon, render_page_link_icon};

use tomet_shelf_config::BookConfig;
use tomet_shelf_icons::IconRenderer;

/// Enhances a raw HTML body string by injecting presentation tags:
/// list markers, code blocks, and external links.
pub fn enhance_body_html(
    body_html: &str,
    config: &BookConfig,
    base_path: &str,
    icon_renderer: Option<&dyn IconRenderer>,
) -> String {
    let html =
        marker::enhance_list_markers(body_html, &config.ui.markers, base_path, icon_renderer);
    let html = code::enhance_code_blocks(&html, Some(&config.book.lang), base_path);
    external_links::enhance_external_links(&html, &config.ui.links, base_path, icon_renderer)
}
