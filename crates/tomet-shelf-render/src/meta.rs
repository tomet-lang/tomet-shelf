use crate::html::escape_html;
use tomet_shelf_document::DocIcon;
use tomet_shelf_icons::{IconRenderer, IconRequest};
use unicode_segmentation::UnicodeSegmentation;

/// Formats a plain text or emoji icon string according to avatar display rules:
/// - 1 char -- single large glyph (`avatar-text-1`)
/// - 2 chars -- horizontally auto-scaled without wrapping (`avatar-text-2`)
/// - 3 chars -- 2x2 grid with 3rd item centered on bottom row (`avatar-grid avatar-grid-3`)
/// - 4 chars -- 2x2 grid (`avatar-grid avatar-grid-4`)
/// - 5 or more chars -- initials (first 2 graphemes rendered with `avatar-text-2 avatar-initials`)
pub fn format_text_icon(s: &str) -> String {
    let graphemes: Vec<&str> = s.graphemes(true).collect();
    match graphemes.len() {
        0 => String::new(),
        1 => {
            format!(
                r#"<span class="avatar-text avatar-text-1">{}</span>"#,
                escape_html(graphemes[0])
            )
        }
        2 => {
            let cells = graphemes
                .iter()
                .map(|g| format!(r#"<span class="avatar-cell">{}</span>"#, escape_html(g)))
                .collect::<Vec<_>>()
                .join("");
            format!(r#"<span class="avatar-text avatar-grid avatar-grid-2">{cells}</span>"#)
        }
        3 => {
            let cells = graphemes
                .iter()
                .map(|g| format!(r#"<span class="avatar-cell">{}</span>"#, escape_html(g)))
                .collect::<Vec<_>>()
                .join("");
            format!(r#"<span class="avatar-text avatar-grid avatar-grid-3">{cells}</span>"#)
        }
        4 => {
            let cells = graphemes
                .iter()
                .map(|g| format!(r#"<span class="avatar-cell">{}</span>"#, escape_html(g)))
                .collect::<Vec<_>>()
                .join("");
            format!(r#"<span class="avatar-text avatar-grid avatar-grid-4">{cells}</span>"#)
        }
        _ => {
            let initials = graphemes[..2].join("");
            format!(
                r#"<span class="avatar-text avatar-text-2 avatar-initials" title="{}">{}</span>"#,
                escape_html(s),
                escape_html(&initials)
            )
        }
    }
}

pub fn format_text_icon_link(s: &str) -> String {
    let graphemes: Vec<&str> = s.graphemes(true).collect();
    match graphemes.len() {
        0 => String::new(),
        1 => {
            let g = graphemes[0];
            let safe_g = escape_html(g);
            let cls = if g.is_ascii() {
                "tm-link-icon tm-link-icon-text"
            } else {
                "tm-link-icon tm-link-icon-emoji"
            };
            format!(r#"<span class="{cls}" aria-hidden="true">{safe_g}</span>"#)
        }
        _ => {
            let initials = graphemes[..2.min(graphemes.len())].join("");
            let safe_init = escape_html(&initials);
            format!(
                r#"<span class="tm-link-icon tm-link-icon-text" aria-hidden="true">{safe_init}</span>"#
            )
        }
    }
}

/// Renders a structured `DocIcon` into an HTML representation for page headers/chrome.
pub fn render_page_icon(
    icon: &DocIcon,
    base_path: &str,
    renderer: Option<&dyn IconRenderer>,
) -> String {
    match icon {
        DocIcon::Named { pkg, name } => {
            let req = IconRequest {
                name,
                pkg,
                base_path,
                inline: true,
                class: Some("tm-doc-icon"),
                args: None,
            };
            if let Some(r) = renderer {
                if let Some(html) = r.render(&req) {
                    return html;
                }
            } else if let Some(r) = tomet_shelf_icons::default_icon_renderer() {
                if let Some(html) = r.render(&req) {
                    return html;
                }
            }
            let safe_name = escape_html(name);
            let safe_pkg = escape_html(pkg);
            format!(
                r#"<svg class="tm-doc-icon" data-pkg="{safe_pkg}" data-icon="{safe_name}" aria-hidden="true"><use href="{base_path}/icons/{safe_pkg}.svg#{safe_name}"></use></svg>"#
            )
        }
        DocIcon::Image { src } => {
            let safe_src = escape_html(src);
            format!(r#"<img class="tm-doc-icon-img" src="{safe_src}" alt="icon" loading="lazy" />"#)
        }
        DocIcon::Text(text) => format_text_icon(text),
    }
}

/// Renders a structured `DocIcon` into a small link prefix HTML icon.
pub fn render_page_link_icon(
    icon: &DocIcon,
    base_path: &str,
    renderer: Option<&dyn IconRenderer>,
) -> String {
    match icon {
        DocIcon::Named { pkg, name } => {
            let req = IconRequest {
                name,
                pkg,
                base_path,
                inline: true,
                class: Some("tm-link-icon"),
                args: None,
            };
            if let Some(r) = renderer {
                if let Some(html) = r.render(&req) {
                    return html;
                }
            } else if let Some(r) = tomet_shelf_icons::default_icon_renderer() {
                if let Some(html) = r.render(&req) {
                    return html;
                }
            }
            let safe_name = escape_html(name);
            let safe_pkg = escape_html(pkg);
            format!(
                r#"<svg class="tm-link-icon tm-link-icon-{safe_pkg}" aria-hidden="true"><use href="{base_path}/icons/{safe_pkg}.svg#{safe_name}"></use></svg>"#
            )
        }
        DocIcon::Image { src } => {
            let safe_src = escape_html(src);
            format!(
                r#"<img class="tm-link-icon tm-link-favicon tm-link-icon-img" src="{safe_src}" alt="" width="14" height="14" loading="lazy" decoding="async" />"#
            )
        }
        DocIcon::Text(text) => format_text_icon_link(text),
    }
}
