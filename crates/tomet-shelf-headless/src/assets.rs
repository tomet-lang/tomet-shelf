use anyhow::{Context, Result};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

use tomet_shelf_config::BookConfig;

pub fn content_version(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// A static asset file (CSS, JS, SVG, etc.) to be written to `out_dir` during
/// static builds and served from memory by the dev server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticAsset {
    /// Relative path without leading slash (e.g. `"shelf.css"`, `"icons/lucide.svg"`).
    pub path: &'static str,
    /// HTTP Content-Type header (e.g. `"text/css; charset=utf-8"`).
    pub mime: &'static str,
    /// Raw file content.
    pub content: &'static str,
}

pub fn write_static_assets(
    out_dir: &Path,
    src_dir: &Path,
    config: &BookConfig,
    theme_assets: &[StaticAsset],
) -> Result<()> {
    fs::create_dir_all(out_dir)?;

    for asset in theme_assets {
        let rel = asset.path.trim_start_matches('/');
        write_if_changed(&out_dir.join(rel), asset.content)
            .with_context(|| format!("Failed to write {}", asset.path))?;
    }

    for css_rel in &config.ui.custom_css {
        let clean_rel = css_rel.trim_start_matches('/');
        let src_file = src_dir.join(clean_rel);
        if src_file.exists() && src_file.is_file() {
            let dest_file = out_dir.join(clean_rel);
            if let Some(parent) = dest_file.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = fs::copy(&src_file, &dest_file);
        }
    }

    Ok(())
}

pub fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.exists()
        && let Ok(existing) = fs::read_to_string(path)
        && existing == content
    {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(true)
}
