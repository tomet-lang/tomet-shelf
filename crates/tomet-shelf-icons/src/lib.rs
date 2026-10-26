//! Standard icon provider and inline SVG renderer for `tomet-shelf`.
//!
//! Provides inline SVG rendering for Lucide and Simple Icons, removing the need
//! for external SVG sprite sheets and resolving browser Shadow DOM / CORS quirks.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, LazyLock, RwLock};
use tomet::ast::Value;

/// Request parameters for rendering an icon.
#[derive(Debug, Clone)]
pub struct IconRequest<'a> {
    /// The name of the icon or media path (e.g. "star", "github", "logo.png").
    pub name: &'a str,
    /// Icon package or namespace (e.g. "lucide", "simple", "stela").
    pub pkg: &'a str,
    /// Base path for URL prefixes.
    pub base_path: &'a str,
    /// Whether the icon is rendered in an inline context (like body text) or block context.
    pub inline: bool,
    /// Optional CSS class name to apply to the root element (e.g. "tm-doc-icon", "tm-link-icon").
    pub class: Option<&'a str>,
    /// Additional arguments passed to the `@doc.icon(...)` element, if any.
    pub args: Option<&'a Value>,
}

/// Pluggable icon renderer trait implemented by icon libraries (like `tomet-shelf-icons`)
/// or custom shelf engines (like `stela-shelf`).
pub trait IconRenderer: Send + Sync + Debug {
    /// Returns true if this renderer knows how to handle the given icon in `pkg`.
    fn has_icon(&self, pkg: &str, name: &str) -> bool;

    /// Render an icon into an HTML string (e.g. inline `<svg>`, `<img>`, or custom container).
    /// Returns `None` if the renderer cannot handle this icon or delegates to fallback.
    fn render(&self, req: &IconRequest) -> Option<String>;

    /// Render a marker icon (e.g. for checklist/task markers).
    fn render_marker(&self, icon: &str, pkg: &str, base_path: &str) -> Option<String> {
        self.render(&IconRequest {
            name: icon,
            pkg,
            base_path,
            inline: true,
            class: Some("tm-doc-icon"),
            args: None,
        })
    }

    /// Render a link prefix icon.
    fn render_link(&self, icon: &str, pkg: &str, base_path: &str) -> Option<String> {
        self.render(&IconRequest {
            name: icon,
            pkg,
            base_path,
            inline: true,
            class: Some("tm-link-icon"),
            args: None,
        })
    }
}

static GLOBAL_ICON_RENDERER: RwLock<Option<Arc<dyn IconRenderer>>> = RwLock::new(None);

/// Sets the global default icon renderer (e.g. `StandardIconRenderer` from `tomet-shelf-icons`).
pub fn set_default_icon_renderer(renderer: Arc<dyn IconRenderer>) {
    if let Ok(mut lock) = GLOBAL_ICON_RENDERER.write() {
        *lock = Some(renderer);
    }
}

/// Retrieves the global default icon renderer if one has been registered.
pub fn default_icon_renderer() -> Option<Arc<dyn IconRenderer>> {
    GLOBAL_ICON_RENDERER
        .read()
        .ok()
        .and_then(|lock| lock.clone())
}

pub const LUCIDE_SPRITE: &str = include_str!("../assets/icons/lucide/sprite.svg");
pub const SIMPLE_SPRITE: &str = include_str!("../assets/icons/simple/sprite.svg");

#[derive(Debug, Clone, Copy)]
pub struct SymbolData {
    pub view_box: &'static str,
    pub inner_xml: &'static str,
}

fn parse_symbols(sprite: &'static str) -> HashMap<&'static str, SymbolData> {
    let mut map = HashMap::new();
    let needle = "<symbol ";
    let mut pos = 0;
    while let Some(start) = sprite[pos..].find(needle) {
        let abs_start = pos + start;
        let tag_end = match sprite[abs_start..].find('>') {
            Some(end) => abs_start + end,
            None => break,
        };
        let header = &sprite[abs_start..tag_end];

        let id = match header.find("id=\"") {
            Some(idx) => {
                let s = &header[idx + 4..];
                match s.find('"') {
                    Some(end) => &s[..end],
                    None => "",
                }
            }
            None => "",
        };

        let view_box = match header.find("viewBox=\"") {
            Some(idx) => {
                let s = &header[idx + 9..];
                match s.find('"') {
                    Some(end) => &s[..end],
                    None => "0 0 24 24",
                }
            }
            None => "0 0 24 24",
        };

        let close_tag = "</symbol>";
        let content_end = match sprite[tag_end + 1..].find(close_tag) {
            Some(idx) => tag_end + 1 + idx,
            None => break,
        };
        let inner_xml = sprite[tag_end + 1..content_end].trim();

        if !id.is_empty() {
            map.insert(
                id,
                SymbolData {
                    view_box,
                    inner_xml,
                },
            );
        }
        pos = content_end + close_tag.len();
    }
    map
}

static LUCIDE_MAP: LazyLock<HashMap<&'static str, SymbolData>> =
    LazyLock::new(|| parse_symbols(LUCIDE_SPRITE));
static SIMPLE_MAP: LazyLock<HashMap<&'static str, SymbolData>> =
    LazyLock::new(|| parse_symbols(SIMPLE_SPRITE));

fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn arg_str<'a>(args: Option<&'a Value>, key: &str) -> Option<&'a str> {
    let Value::Map(entries) = args? else {
        return None;
    };
    entries
        .iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        })
}

fn wrap_icon(html: String, req: &IconRequest) -> String {
    if req.class == Some("tm-link-icon") {
        return html;
    }
    let has_circle = arg_str(req.args, "shape") == Some("circle");
    let inner = if has_circle {
        format!(r#"<span class="tm-icon-wrapper tm-icon-circle">{html}</span>"#)
    } else {
        html
    };

    if req.inline {
        inner
    } else {
        format!(r#"<div class="tm-doc-icon-block">{inner}</div>"#)
    }
}

/// Standard icon renderer that renders Lucide and Simple Icons as inline SVG,
/// or images as `<img>` tags.
#[derive(Debug, Clone, Default)]
pub struct StandardIconRenderer;

impl StandardIconRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl IconRenderer for StandardIconRenderer {
    fn has_icon(&self, pkg: &str, name: &str) -> bool {
        match pkg {
            "lucide" => LUCIDE_MAP.contains_key(name),
            "simple" => SIMPLE_MAP.contains_key(name),
            _ => false,
        }
    }

    fn render(&self, req: &IconRequest) -> Option<String> {
        let base_class = req.class.unwrap_or("tm-doc-icon");

        // 1. Check if name is an image path
        if req.name.ends_with(".png")
            || req.name.ends_with(".jpg")
            || req.name.ends_with(".jpeg")
            || req.name.ends_with(".webp")
            || (req.name.ends_with(".svg") && req.name.contains('/'))
        {
            let safe_src = escape_html(req.name);
            let img =
                format!(r#"<img class="{base_class} tm-doc-icon-img" src="{safe_src}" alt="" />"#);
            return Some(wrap_icon(img, req));
        }

        // 2. Look up symbol in specified package
        let (symbol, is_fill) = match req.pkg {
            "lucide" => (LUCIDE_MAP.get(req.name)?, false),
            "simple" => (SIMPLE_MAP.get(req.name)?, true),
            _ => return None,
        };

        let (stroke_attrs, fill_attr, extra_class) = if is_fill {
            ("", r#"fill="currentColor" stroke="none""#, "tm-icon-simple")
        } else {
            (
                r#"stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round""#,
                r#"fill="none""#,
                "tm-icon-lucide",
            )
        };

        let safe_name = escape_html(req.name);
        let svg = format!(
            r#"<svg class="{base_class} {extra_class}" data-icon="{safe_name}" viewBox="{}" {} {} aria-hidden="true">{}</svg>"#,
            symbol.view_box, fill_attr, stroke_attrs, symbol.inner_xml,
        );

        Some(wrap_icon(svg, req))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_parsing_counts() {
        assert!(
            LUCIDE_MAP.len() > 1000,
            "Lucide map size: {}",
            LUCIDE_MAP.len()
        );
        assert!(
            SIMPLE_MAP.len() > 50,
            "Simple map size: {}",
            SIMPLE_MAP.len()
        );
        assert!(LUCIDE_MAP.contains_key("star"));
        assert!(LUCIDE_MAP.contains_key("cake"));
        assert!(SIMPLE_MAP.contains_key("github"));
    }

    #[test]
    fn test_render_lucide_inline_svg() {
        let renderer = StandardIconRenderer::new();
        let req = IconRequest {
            name: "star",
            pkg: "lucide",
            base_path: "",
            inline: true,
            class: None,
            args: None,
        };
        let html = renderer.render(&req).expect("star icon should render");
        assert!(html.starts_with("<svg class=\"tm-doc-icon tm-icon-lucide\""));
        assert!(html.contains("data-icon=\"star\""));
        assert!(html.contains("stroke=\"currentColor\""));
    }

    #[test]
    fn test_render_simple_icon() {
        let renderer = StandardIconRenderer::new();
        let req = IconRequest {
            name: "github",
            pkg: "simple",
            base_path: "",
            inline: true,
            class: None,
            args: None,
        };
        let html = renderer.render(&req).expect("github icon should render");
        assert!(html.starts_with("<svg class=\"tm-doc-icon tm-icon-simple\""));
        assert!(html.contains("fill=\"currentColor\""));
    }

    #[test]
    fn test_render_with_circle_shape() {
        let renderer = StandardIconRenderer::new();
        let args = Value::Map(vec![(
            "shape".to_string(),
            Value::String("circle".to_string()),
        )]);
        let req = IconRequest {
            name: "star",
            pkg: "lucide",
            base_path: "",
            inline: true,
            class: None,
            args: Some(&args),
        };
        let html = renderer
            .render(&req)
            .expect("star icon with circle should render");
        assert!(html.starts_with("<span class=\"tm-icon-wrapper tm-icon-circle\"><svg"));
    }
}
