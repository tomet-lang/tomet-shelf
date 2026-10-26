use crate::icon::DocIcon;
use serde_json::{Map, Value};
use std::path::Path;

use super::MetaCtx;

/// Extract media reference target string (file name or path) from a metadata value.
pub(super) fn extract_media_target(val: &Value) -> Option<String> {
    match val {
        Value::String(s) => Some(s.clone()),
        Value::Object(map) => {
            let el_name = map.get("element").and_then(|v| v.as_str())?;
            if el_name == "link" || el_name == "embed" || el_name == "ref" {
                let args = map.get("args")?;
                match args {
                    Value::String(s) => Some(s.clone()),
                    Value::Object(arg_map) => arg_map
                        .get("target")
                        .or_else(|| arg_map.get("ref"))
                        .or_else(|| arg_map.get("path"))
                        .or_else(|| arg_map.get("src"))
                        .or_else(|| arg_map.get("url"))
                        .or_else(|| arg_map.get("href"))
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Helper to test if a string represents an image target (by file extension or scheme).
pub(super) fn is_image_target(s: &str) -> bool {
    let s_clean = s
        .split('?')
        .next()
        .unwrap_or(s)
        .split('#')
        .next()
        .unwrap_or(s);
    let ext = Path::new(s_clean)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    matches!(
        ext.as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "avif" | "ico")
    )
}

/// Extracts document icon as structured `DocIcon` and optional image URL for optimization.
///
/// Supports:
/// - Named icon syntax (`"lucide:star"`, `"simple:github"`, etc.) -> `DocIcon::Named`
/// - Embedded elements (`@embed("avatar.png")`, `@link(...)`) -> `DocIcon::Image`
/// - Image file paths or URLs (`"avatar.png"`, `"https://..."`) -> `DocIcon::Image`
/// - Plain string / emoji (`"🎂"`, `"cat"`) -> `DocIcon::Text`
/// - `@doc.icon(...)` elements -> `DocIcon::Named`
pub(super) fn extract_icon(
    cx: &MetaCtx,
    map: &Map<String, Value>,
) -> (Option<DocIcon>, Option<String>) {
    let Some(val) = map.get("icon") else {
        return (None, None);
    };

    match val {
        Value::String(s) => {
            let s_trimmed = s.trim();
            if let Some((pkg, name)) = s_trimmed.split_once(':') {
                return (
                    Some(DocIcon::Named {
                        pkg: pkg.to_string(),
                        name: name.to_string(),
                    }),
                    None,
                );
            }

            if is_image_target(s) || s.starts_with("http://") || s.starts_with("https://") {
                let resolved = cx.media_path(s);
                (
                    Some(DocIcon::Image {
                        src: resolved.clone(),
                    }),
                    Some(resolved),
                )
            } else {
                (Some(DocIcon::Text(s_trimmed.to_string())), None)
            }
        }
        Value::Object(obj) => {
            let el_name = obj.get("element").and_then(|v| v.as_str());
            if el_name == Some("embed") || el_name == Some("link") || el_name == Some("ref") {
                if let Some(target) = extract_media_target(val) {
                    let resolved = cx.media_path(&target);
                    return (
                        Some(DocIcon::Image {
                            src: resolved.clone(),
                        }),
                        Some(resolved),
                    );
                }
            }
            if el_name == Some("doc.icon") {
                let args = obj.get("args");
                let name = match args {
                    Some(Value::Object(m)) => {
                        m.get("name").and_then(|v| v.as_str()).unwrap_or_default()
                    }
                    Some(Value::String(s)) => s.as_str(),
                    _ => "",
                };
                let pkg = match args {
                    Some(Value::Object(m)) => {
                        m.get("pkg").and_then(|v| v.as_str()).unwrap_or("lucide")
                    }
                    _ => "lucide",
                };
                return (
                    Some(DocIcon::Named {
                        pkg: pkg.to_string(),
                        name: name.to_string(),
                    }),
                    None,
                );
            }
            (None, None)
        }
        _ => (None, None),
    }
}
