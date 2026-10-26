//! Renders `@doc.icon("name", pkg:"lucide")` via a pluggable [`IconRenderer`].
//!
//! Tomet resolves this element but deliberately never draws it, by design: an
//! external renderer is meant to implement the notation. This module connects
//! the element to [`IconRenderer`] (wired in as a
//! `tomet_html::RenderOptions::custom_element` hook).
//!
//! If no [`IconRenderer`] is configured or an unknown icon is requested,
//! a visible placeholder is rendered so typos in documents are visible.

use std::sync::Arc;
use tomet::ast::Value;
use tomet::html::{CustomElementCtx, CustomElementRenderer};
use tomet_shelf_icons::{IconRenderer, IconRequest};

use crate::html::escape_html;

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

/// A visible stand-in for an icon that cannot be drawn.
fn placeholder(name: &str, pkg: &str, inline: bool) -> String {
    let tag = if inline { "span" } else { "div" };
    format!(
        "<{tag} class=\"tm-doc-icon tm-doc-icon-missing\" title=\"unknown icon: {pkg}/{name}\">?</{tag}>"
    )
}

/// Renders a `@doc.icon` custom element into HTML.
pub fn render_doc_icon(
    ctx: CustomElementCtx,
    base_path: &str,
    renderer: Option<&dyn IconRenderer>,
) -> Option<String> {
    if ctx.kind != "doc.icon" {
        return None;
    }

    let args = ctx.normalized_args();
    let name = arg_str(args.as_ref(), "name").unwrap_or_default();
    let pkg = arg_str(args.as_ref(), "pkg").unwrap_or("lucide");
    let name_attr = escape_html(name);
    let pkg_attr = escape_html(pkg);

    let req = IconRequest {
        name,
        pkg,
        base_path,
        inline: ctx.inline,
        class: Some("tm-doc-icon"),
        args: args.as_ref(),
    };

    if let Some(r) = renderer {
        if let Some(html) = r.render(&req) {
            return Some(html);
        }
        return Some(placeholder(&name_attr, &pkg_attr, ctx.inline));
    } else if let Some(r) = tomet_shelf_icons::default_icon_renderer() {
        if let Some(html) = r.render(&req) {
            return Some(html);
        }
        return Some(placeholder(&name_attr, &pkg_attr, ctx.inline));
    }

    Some(format!(
        r#"<svg class="tm-doc-icon" data-icon="{name_attr}" data-pkg="{pkg_attr}" aria-hidden="true"><use href="{base_path}/icons/{pkg_attr}.svg#{name_attr}"></use></svg>"#
    ))
}

/// Creates a `CustomElementRenderer` that handles `@doc.icon` elements.
pub fn create_doc_icon_custom_element(
    base_path: &str,
    renderer: Option<Arc<dyn IconRenderer>>,
) -> CustomElementRenderer {
    let base = base_path.to_string();
    CustomElementRenderer::new(move |ctx| render_doc_icon(ctx, &base, renderer.as_deref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockRenderer;
    impl IconRenderer for MockRenderer {
        fn has_icon(&self, pkg: &str, name: &str) -> bool {
            pkg == "lucide" && name == "star"
        }
        fn render(&self, req: &IconRequest) -> Option<String> {
            if self.has_icon(req.pkg, req.name) {
                Some(format!("<svg class=\"mock-icon\">{}</svg>", req.name))
            } else {
                None
            }
        }
    }

    fn args(name: &str, pkg: Option<&str>) -> Value {
        let mut entries = vec![("name".to_string(), Value::String(name.to_string()))];
        if let Some(pkg) = pkg {
            entries.push(("pkg".to_string(), Value::String(pkg.to_string())));
        }
        Value::Map(entries)
    }

    fn ctx<'a>(kind: &'a str, args: &'a Value, inline: bool) -> CustomElementCtx<'a> {
        CustomElementCtx::new(kind, None, inline, || Some(args.clone()))
    }

    #[test]
    fn ignores_kinds_other_than_doc_icon() {
        let a = args("star", Some("lucide"));
        assert_eq!(render_doc_icon(ctx("doc.index", &a, true), "", None), None);
    }

    #[test]
    fn renders_with_mock_renderer() {
        let renderer = MockRenderer;
        let a = args("star", Some("lucide"));
        let html = render_doc_icon(ctx("doc.icon", &a, true), "", Some(&renderer)).unwrap();
        assert_eq!(html, "<svg class=\"mock-icon\">star</svg>");
    }

    #[test]
    fn an_unknown_icon_name_is_a_visible_placeholder_not_nothing() {
        let renderer = MockRenderer;
        let a = args("this-icon-does-not-exist", Some("lucide"));
        let html = render_doc_icon(ctx("doc.icon", &a, true), "", Some(&renderer)).unwrap();
        assert!(html.contains("tm-doc-icon-missing"));
        assert!(html.contains("this-icon-does-not-exist"));
    }
}
