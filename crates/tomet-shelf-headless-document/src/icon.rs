use serde::{Deserialize, Serialize};

/// Structured representation of a document icon.
///
/// Documents never produce HTML strings for icons. Instead, they produce
/// structured `DocIcon` variants, delegating presentation markup to `tomet-shelf-render`
/// or shelf UI engines (e.g. Leptos, Svelte, Web Components).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocIcon {
    /// A named vector icon from an icon package (e.g. "star" from "lucide", "github" from "simple").
    Named { pkg: String, name: String },
    /// An image reference (resolved vault media path or external URL).
    Image { src: String },
    /// Plain text or emoji (e.g. "🎂", "API").
    Text(String),
}
