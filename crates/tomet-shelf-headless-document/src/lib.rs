//! Document parsing, AST transformation, and metadata extraction for `tomet-shelf`.

pub mod html;
pub mod icon;
pub mod links;
pub mod meta;
pub mod toc;

pub use icon::DocIcon;
pub use tomet_shelf_route as route;
pub use tomet_shelf_route as slug;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

pub use links::strip_doc_extension;
pub use meta::{HeroChipItem, InfoboxRowItem};
pub use route::{
    DocRouteInput, RouteTable, clean_doc_slug, clean_segment, normalize_path, normalize_path_str,
};
pub use toc::{SectionTab, TocItem};

use toc::{build_section_tabs, toc_from_outline};
use tomet_shelf_config::BookConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDoc {
    pub slug: String,
    pub rel_path: String,
    pub source_path: String,
    pub title: String,
    pub section: Option<String>,
    pub kind: Option<String>,
    pub primary_color: Option<String>,
    pub icon: Option<DocIcon>,
    pub icon_image_url: Option<String>,
    pub banner_url: Option<String>,
    pub banner_original_url: Option<String>,
    pub banner_y: Option<f64>,
    pub images: Vec<String>,
    pub original_images: Vec<String>,
    pub hero_chips: Vec<HeroChipItem>,
    pub infobox_rows: Vec<InfoboxRowItem>,
    pub has_data: bool,
    pub toc: Vec<TocItem>,
    pub section_tabs: Vec<SectionTab>,
    pub body_html: String,
    /// Pages this one links to, as slugs. The build reverses these into the
    /// backlinks each page shows.
    pub outgoing: Vec<String>,
}

/// Extract the `::config`/`::settings` blocks of a workspace-wide document.
pub fn extract_workspace_config_blocks(workspace_cfg_src: &str) -> Vec<tomet::ast::Block> {
    let Ok(cfg_doc) = tomet::parser::parse_document(workspace_cfg_src) else {
        return Vec::new();
    };

    cfg_doc
        .blocks
        .into_iter()
        .filter(|block| match block {
            tomet::ast::Block::Element(el) => {
                let kind = tomet::semantics::classify_std_lenient(el);
                kind.as_str() == "config" || kind.as_str() == "settings"
            }
            _ => false,
        })
        .collect()
}

/// Prepend pre-parsed workspace-wide config blocks so shared macros are in scope.
pub fn inject_workspace_config_blocks(
    doc: &mut tomet::ast::Document,
    prefix_blocks: &[tomet::ast::Block],
) {
    if !prefix_blocks.is_empty() {
        let mut new_blocks = prefix_blocks.to_vec();
        new_blocks.append(&mut doc.blocks);
        doc.blocks = new_blocks;
    }
}

/// The page title: `@meta.title`, else the first H1 when the filename says
/// nothing (`index`, `readme`), else the filename.
fn resolve_title(
    rel_path: &str,
    meta_json: Option<&serde_json::Value>,
    first_h1: Option<String>,
) -> String {
    let stem = Path::new(rel_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    let meta_title = meta_json
        .and_then(|m| m.get("title"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    if let Some(meta_title) = meta_title {
        return meta_title;
    }

    let is_generic_stem = stem.eq_ignore_ascii_case("index")
        || stem.eq_ignore_ascii_case("readme")
        || stem == "Untitled";

    if is_generic_stem {
        first_h1.unwrap_or(stem)
    } else {
        stem
    }
}

/// Read a document far enough to know what it is.
///
/// The build has to decide what the site publishes before it resolves a
/// single link, since a link to a document the site leaves out has to come
/// out broken. That decision needs the `::kind`, and the `::kind` needs a
/// parse -- so the parse is done once, here, and handed on.
pub fn parse_tomet_document(
    source: &str,
    rel_path: &str,
) -> Result<(tomet::ast::Document, Option<String>)> {
    let doc = tomet::parser::parse_document(source)
        .map_err(|e| anyhow::anyhow!("Failed to parse {rel_path}: {e}"))?;
    let kind = tomet::declaration::document_kind(&doc).map(|s| s.to_string());
    Ok((doc, kind))
}

pub fn process_tomet_document(
    source: &str,
    rel_path: &str,
    abs_path: Option<&Path>,
    config: &BookConfig,
    vault_index: &tomet::links::VaultLinkIndex,
    workspace_cfg_src: Option<&str>,
    unpublished: &HashSet<String>,
) -> Result<ProcessedDoc> {
    let (doc, _) = parse_tomet_document(source, rel_path)?;
    process_parsed_document(
        doc,
        rel_path,
        abs_path,
        config,
        vault_index,
        workspace_cfg_src,
        unpublished,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn process_parsed_document(
    doc: tomet::ast::Document,
    rel_path: &str,
    abs_path: Option<&Path>,
    config: &BookConfig,
    vault_index: &tomet::links::VaultLinkIndex,
    workspace_cfg_src: Option<&str>,
    unpublished: &HashSet<String>,
    route_table: Option<&RouteTable>,
) -> Result<ProcessedDoc> {
    let blocks = workspace_cfg_src
        .map(extract_workspace_config_blocks)
        .unwrap_or_default();
    process_parsed_document_with_blocks(
        doc,
        rel_path,
        abs_path,
        config,
        vault_index,
        &blocks,
        unpublished,
        route_table,
    )
}

/// Turn a parsed document into the page it becomes using pre-parsed workspace config blocks.
#[allow(clippy::too_many_arguments)]
pub fn process_parsed_document_with_blocks(
    doc: tomet::ast::Document,
    rel_path: &str,
    abs_path: Option<&Path>,
    config: &BookConfig,
    vault_index: &tomet::links::VaultLinkIndex,
    workspace_cfg_blocks: &[tomet::ast::Block],
    unpublished: &HashSet<String>,
    route_table: Option<&RouteTable>,
) -> Result<ProcessedDoc> {
    process_parsed_document_with_custom_element(
        doc,
        rel_path,
        abs_path,
        config,
        vault_index,
        workspace_cfg_blocks,
        unpublished,
        route_table,
        None,
    )
}

/// Turn a parsed document into the page it becomes using pre-parsed workspace config blocks
/// and an optional custom element renderer (e.g. for `@doc.icon`).
#[allow(clippy::too_many_arguments)]
pub fn process_parsed_document_with_custom_element(
    mut doc: tomet::ast::Document,
    rel_path: &str,
    abs_path: Option<&Path>,
    config: &BookConfig,
    vault_index: &tomet::links::VaultLinkIndex,
    workspace_cfg_blocks: &[tomet::ast::Block],
    unpublished: &HashSet<String>,
    route_table: Option<&RouteTable>,
    custom_element: Option<tomet::html::CustomElementRenderer>,
) -> Result<ProcessedDoc> {
    // 1. Bring in the vault-wide config (e.g. default.config.tmt)
    inject_workspace_config_blocks(&mut doc, workspace_cfg_blocks);

    // 2. Prepare expressions and macros via Tomet Vault
    let vault_root = abs_path.and_then(|p| p.parent()).unwrap_or(Path::new("."));
    let vault = tomet::Vault::discover(vault_root);
    let prepared = vault.prepare(&mut doc, Path::new(rel_path));
    for u in &prepared.unresolved {
        tracing::debug!(
            "{rel_path}: unresolved expression: ${{{}}} ({})",
            u.source,
            u.reason
        );
    }

    // 3. Collect outgoing links, then resolve them
    let from_path = Path::new(rel_path);
    let slug = route_table
        .map(|rt| rt.get_or_clean(rel_path))
        .unwrap_or_else(|| clean_doc_slug(rel_path));
    let mut outgoing = links::outgoing_page_slugs(
        &doc,
        from_path,
        &slug,
        vault_index,
        unpublished,
        route_table,
    );

    let mode = tomet::transform::TargetMode::WebSlug {
        url_prefix: config.build.full_url_prefix(),
        asset_prefix: config.build.full_asset_prefix(),
    };
    tomet::transform::resolve_document_links(
        &mut doc,
        Some(from_path),
        |target, from| {
            vault_index
                .resolve_ref(target, from)
                // A page the site does not publish is not a page. Resolving
                // it would print a link to a file that was never written.
                .filter(|p| !unpublished.contains(&normalize_path(p)))
                .map(|p| {
                    let norm = normalize_path(p);
                    if links::strip_doc_extension(&norm) != norm {
                        let clean_slug = route_table
                            .map(|rt| rt.get_or_clean(&norm))
                            .unwrap_or_else(|| clean_doc_slug(&norm));
                        std::path::PathBuf::from(format!("{clean_slug}.tmt"))
                    } else {
                        p.to_path_buf()
                    }
                })
        },
        &mode,
    );

    // 4. Metadata and kind
    let kind = tomet::declaration::document_kind(&doc).map(|s| s.to_string());
    let raw_meta = tomet::declaration::document_meta(&doc);
    let meta_json = raw_meta.as_ref().map(tomet::semantics::value_to_json);

    // 5. Render, and take the heading outline the renderer reports
    let render_opts = tomet::html::RenderOptions {
        number_headings: true,
        auto_slug_headings: true,
        lang: Some(config.book.lang.clone()),
        custom_element,
        ..Default::default()
    };
    let (body_html, outline) = tomet::html::render_body_with_outline(&doc, &render_opts);

    // 6. Table of contents, title, and the sticky-tab tree
    let (first_h1, mut toc) = toc_from_outline(&outline);
    let title = resolve_title(rel_path, meta_json.as_ref(), first_h1);

    // The title already stands at the top of the page; no need to repeat it.
    if let Some(first) = toc.first()
        && first.level == 1
        && first.text == title
    {
        toc.remove(0);
    }
    let section_tabs = build_section_tabs(&toc);

    // 7. Where the page lives: "30-39 Knowledge/01 Dev/a.tmt" -> section, slug
    let section = Path::new(rel_path)
        .parent()
        .and_then(|p| p.iter().next())
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    // 8. The page's chrome: colours, banner, chips, infobox
    let mut props = meta::extract(
        meta_json.as_ref(),
        kind.as_deref(),
        from_path,
        vault_index,
        config,
        route_table,
    );

    // `icon: @doc.icon("cake")` fallback if not parsed via meta_json
    if props.icon.is_none()
        && let Some(tomet::ast::Value::Map(entries)) = raw_meta.as_ref()
        && let Some((_, tomet::ast::Value::Element(el))) = entries.iter().find(|(k, _)| k == "icon")
    {
        let name = el.sigil.name().map(|n| n.to_string()).unwrap_or_default();
        if name == "doc.icon" {
            let bindings = tomet::semantics::Bindings::default();
            let args = tomet::semantics::normalized_element_args_in(el, &bindings);
            if let Some(tomet::ast::Value::Map(arg_map)) = args.as_ref() {
                let icon_name = arg_map
                    .iter()
                    .find(|(k, _)| k == "name")
                    .and_then(|(_, v)| match v {
                        tomet::ast::Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let pkg = arg_map
                    .iter()
                    .find(|(k, _)| k == "pkg")
                    .and_then(|(_, v)| match v {
                        tomet::ast::Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "lucide".to_string());
                props.icon = Some(DocIcon::Named {
                    pkg,
                    name: icon_name,
                });
            }
        }
    }

    // Merge links from @meta into body links, dropping self-references and duplicates.
    for linked in std::mem::take(&mut props.linked_slugs) {
        if linked != slug && !outgoing.contains(&linked) {
            outgoing.push(linked);
        }
    }

    let source_path = abs_path
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.to_string());

    let banner_original_url = props.banner_url.clone();
    let original_images = props.images.clone();

    Ok(ProcessedDoc {
        slug,
        rel_path: rel_path.to_string(),
        source_path,
        title,
        section,
        kind,
        has_data: props.has_data(),
        primary_color: props.primary_color,
        icon: props.icon,
        icon_image_url: props.icon_image_url,
        banner_url: props.banner_url,
        banner_original_url,
        banner_y: props.banner_y,
        images: props.images,
        original_images,
        hero_chips: props.hero_chips,
        infobox_rows: props.infobox_rows,
        toc,
        section_tabs,
        body_html,
        outgoing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_title_wins_over_everything() {
        let meta = serde_json::json!({ "title": "From Meta" });
        assert_eq!(
            resolve_title("notes/file.tmt", Some(&meta), Some("From H1".into())),
            "From Meta"
        );
    }

    #[test]
    fn a_named_file_keeps_its_name() {
        assert_eq!(
            resolve_title("notes/Rust.tmt", None, Some("Some Heading".into())),
            "Rust"
        );
    }

    #[test]
    fn a_generic_filename_defers_to_the_first_heading() {
        for stem in ["index", "README"] {
            let path = format!("notes/{stem}.tmt");
            assert_eq!(
                resolve_title(&path, None, Some("Real Title".into())),
                "Real Title"
            );
        }
    }

    #[test]
    fn a_generic_filename_with_no_heading_falls_back_to_the_stem() {
        assert_eq!(resolve_title("notes/index.tmt", None, None), "index");
    }
}

#[cfg(test)]
mod kind_tests {
    use super::*;
    use tomet::links::VaultLinkIndex;

    #[test]
    fn a_document_reports_the_kind_it_declares() {
        let (_, kind) = parse_tomet_document("::kind(config)\n\n#[ x ]\n", "a.tmt").unwrap();
        assert_eq!(kind.as_deref(), Some("config"));

        let (_, none) = parse_tomet_document("#[ x ]\n", "b.tmt").unwrap();
        assert_eq!(none, None);
    }

    #[test]
    fn a_link_to_an_unpublished_document_resolves_to_nothing() {
        let config = BookConfig::default();
        let vault = VaultLinkIndex::from_paths(&["page.tmt".to_string(), "secret.tmt".to_string()]);

        let unpublished: HashSet<String> = ["secret.tmt".to_string()].into_iter().collect();
        let out = process_tomet_document(
            "#[ Page ]\n\n@link(ref:\"secret\")\n",
            "page.tmt",
            None,
            &config,
            &vault,
            None,
            &unpublished,
        )
        .unwrap();

        assert!(
            out.body_html.contains("tm-ref-unresolved"),
            "a link to a page the book leaves out must read as broken: {}",
            out.body_html
        );
        assert!(out.outgoing.is_empty(), "and must not count as a backlink");
    }

    #[test]
    fn the_same_link_resolves_when_the_page_is_published() {
        let config = BookConfig::default();
        let vault = VaultLinkIndex::from_paths(&["page.tmt".to_string(), "secret.tmt".to_string()]);

        let out = process_tomet_document(
            "#[ Page ]\n\n@link(ref:\"secret\")\n",
            "page.tmt",
            None,
            &config,
            &vault,
            None,
            &HashSet::new(),
        )
        .unwrap();

        assert!(
            !out.body_html.contains("tm-ref-unresolved"),
            "{}",
            out.body_html
        );
        assert_eq!(out.outgoing, ["secret"]);
    }

    #[test]
    fn a_page_renders_ruby() {
        let config = BookConfig::default();
        let vault = VaultLinkIndex::from_paths(&["page.tmt".to_string()]);

        let out = process_tomet_document(
            "#[ Page ]\n\n@ruby[漢字](rt:\"かんじ\").\n",
            "page.tmt",
            None,
            &config,
            &vault,
            None,
            &HashSet::new(),
        )
        .unwrap();

        assert!(
            out.body_html.contains("<ruby>漢字<rt>かんじ</rt></ruby>"),
            "{}",
            out.body_html
        );
    }

    #[test]
    fn a_meta_icon_written_as_a_doc_icon_element_resolves_to_named_page_icon() {
        let config = BookConfig::default();
        let vault = VaultLinkIndex::from_paths(&["page.tmt".to_string()]);

        let out = process_tomet_document(
            "::meta{icon: @doc.icon(\"cake\")}\n\n#[ Page ]\n\nbody\n",
            "page.tmt",
            None,
            &config,
            &vault,
            None,
            &HashSet::new(),
        )
        .unwrap();

        assert_eq!(
            out.icon,
            Some(DocIcon::Named {
                pkg: "lucide".to_string(),
                name: "cake".to_string(),
            })
        );
    }

    #[test]
    fn a_meta_icon_supports_raw_text_and_custom_pkg() {
        let config = BookConfig::default();
        let vault = VaultLinkIndex::from_paths(&["page.tmt".to_string()]);

        let out = process_tomet_document(
            "::meta{icon: \"🎂\"}\n\n#[ Page ]\n\nbody\n",
            "page.tmt",
            None,
            &config,
            &vault,
            None,
            &HashSet::new(),
        )
        .unwrap();
        assert_eq!(out.icon, Some(DocIcon::Text("🎂".to_string())));

        let out2 = process_tomet_document(
            "::meta{icon: \"simple:github\"}\n\n#[ Page ]\n\nbody\n",
            "page.tmt",
            None,
            &config,
            &vault,
            None,
            &HashSet::new(),
        )
        .unwrap();
        assert_eq!(
            out2.icon,
            Some(DocIcon::Named {
                pkg: "simple".to_string(),
                name: "github".to_string(),
            })
        );
    }
}
