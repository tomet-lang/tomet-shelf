//! Incremental document loading, change detection, and reconciliation for book builds.

use anyhow::Result;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use tomet_shelf_config::BookConfig;
use tomet_shelf_document::ProcessedDoc;
use tomet_shelf_document::links::parse_link_ref;
use tomet_shelf_document::slug::{
    DocRouteInput, RouteTable, extract_route_metadata, normalize_path,
};
use tomet_shelf_incremental::{
    AffectedPaths, Change, ChangedDoc, FileSignature, ScannedDoc, VaultCache,
    compute_affected_paths, detect_changes, reconcile_vault_state,
};

use super::kinds;
use super::loader::ScannedVault;
use super::{DocFailure, document};
use crate::renderer::RendererFactory;

/// The resolved state produced by incremental processing of a vault.
pub struct IncrementalVaultState {
    pub route_table: RouteTable,
    pub unpublished: HashSet<String>,
    pub processed: Vec<Result<ProcessedDoc, DocFailure>>,
    pub new_cache: VaultCache,
    pub affected: AffectedPaths,
    pub changes: Vec<Change>,
    pub parsed_for_index: Vec<Result<(tomet::ast::Document, Option<String>), DocFailure>>,
}

/// Converts scanned doc files into [`ScannedDoc`] instances with filesystem signatures.
pub fn prepare_scanned_docs(scanned: &ScannedVault) -> Vec<ScannedDoc> {
    scanned
        .doc_files
        .iter()
        .filter_map(|df| {
            let meta = fs::metadata(&df.abs_path).ok()?;
            Some(ScannedDoc {
                rel_path: df.rel_path.clone(),
                abs_path: df.abs_path.clone(),
                signature: FileSignature::from_metadata(&meta),
            })
        })
        .collect()
}

/// Incrementally processes the vault against the previous cache:
/// 1. Compares file signatures to identify unchanged documents (zero I/O).
/// 2. Reads and parses only added and modified documents in parallel.
/// 3. Builds the route table and extracts dependency nodes.
/// 4. Reconciles state into an updated [`VaultCache`] and calculates affected paths via `tomet::deps`.
pub fn process_incremental_vault(
    prev_cache: &VaultCache,
    scanned: &ScannedVault,
    config: &BookConfig,
    _src_dir: &Path,
    renderer_factory: &dyn RendererFactory,
) -> Result<IncrementalVaultState> {
    let scanned_docs = prepare_scanned_docs(scanned);
    let detection = detect_changes(prev_cache, scanned_docs);

    let unchanged_count = detection.unchanged.len();
    let modified_count = detection.modified.len();
    let added_count = detection.added.len();
    let removed_count = detection.removed.len();

    info!(
        "Incremental scan: {} unchanged (skip I/O), {} modified, {} added, {} removed",
        unchanged_count, modified_count, added_count, removed_count
    );

    // Collect all changed files that require parsing
    let mut files_to_parse: Vec<ChangedDoc> = Vec::new();
    files_to_parse.extend(detection.modified.clone());
    files_to_parse.extend(detection.added.clone());

    // Parse added and modified documents in parallel
    struct ParsedUpdate {
        rel_path: String,
        abs_path: PathBuf,
        signature: FileSignature,
        source: String,
        parsed_doc: Result<(tomet::ast::Document, Option<String>), DocFailure>,
        explicit_slug: Option<String>,
        meta_id: Option<String>,
    }

    let parsed_updates: Vec<ParsedUpdate> = files_to_parse
        .into_par_iter()
        .map(|changed| {
            let fail = |error: String| DocFailure {
                rel_path: changed.rel_path.clone(),
                error,
            };

            let source = match fs::read_to_string(&changed.abs_path) {
                Ok(s) => s,
                Err(e) => {
                    return ParsedUpdate {
                        rel_path: changed.rel_path.clone(),
                        abs_path: changed.abs_path,
                        signature: changed.signature,
                        source: String::new(),
                        parsed_doc: Err(fail(format!("could not read file: {e}"))),
                        explicit_slug: None,
                        meta_id: None,
                    };
                }
            };

            let parsed_doc = document::parse_tomet_document(&source, &changed.rel_path)
                .map_err(|e| fail(format!("could not parse document: {e}")));

            let (explicit_slug, meta_id) = match &parsed_doc {
                Ok((doc, _)) => extract_route_metadata(doc),
                Err(_) => (None, None),
            };

            ParsedUpdate {
                rel_path: changed.rel_path,
                abs_path: changed.abs_path,
                signature: changed.signature,
                source,
                parsed_doc,
                explicit_slug,
                meta_id,
            }
        })
        .collect();

    // Determine unpublished documents
    let mut unpublished = HashSet::new();

    // From unchanged entries: if cached entry had no processed doc, it was unpublished
    for entry in &detection.unchanged {
        if entry.processed.is_none() {
            unpublished.insert(entry.rel_path.clone());
        }
    }

    // From parsed updates
    for update in &parsed_updates {
        match &update.parsed_doc {
            Ok((_, kind)) => {
                if !kinds::publishes(kind.as_deref(), &config.build.kinds) {
                    unpublished.insert(update.rel_path.clone());
                }
            }
            Err(_) => {
                unpublished.insert(update.rel_path.clone());
            }
        }
    }

    // Build the global route table combining unchanged and parsed routes
    let mut route_inputs = Vec::new();

    for entry in &detection.unchanged {
        if !unpublished.contains(&entry.rel_path)
            && let Some(p) = &entry.processed
        {
            route_inputs.push(DocRouteInput {
                rel_path: entry.rel_path.as_str(),
                explicit_slug: Some(p.slug.as_str()),
                meta_id: None,
            });
        }
    }

    for update in &parsed_updates {
        if !unpublished.contains(&update.rel_path) && update.parsed_doc.is_ok() {
            route_inputs.push(DocRouteInput {
                rel_path: update.rel_path.as_str(),
                explicit_slug: update.explicit_slug.as_deref(),
                meta_id: update.meta_id.as_deref(),
            });
        }
    }

    let route_table = RouteTable::build(&route_inputs, &config.build)?;

    // Custom element helper for icon rendering
    let custom_element = Some(tomet_shelf_render::create_doc_icon_custom_element(
        config.build.clean_base_path(),
        renderer_factory.icon_renderer(),
    ));

    // Target resolution closure for dependency extraction
    let resolve_target = |from: &str, raw: &str| -> Option<String> {
        let from_p = Path::new(from);
        let parsed_ref = parse_link_ref(raw);
        let clean_target = parsed_ref.target;
        let resolved = scanned
            .vault_index
            .resolve_ref(clean_target, Some(from_p))
            .or_else(|| {
                let stripped = clean_target.trim_start_matches('@');
                if stripped != clean_target {
                    scanned.vault_index.resolve_ref(stripped, Some(from_p))
                } else {
                    None
                }
            });
        resolved.map(normalize_path)
    };

    struct ProcessedUpdate {
        rel_path: String,
        signature: FileSignature,
        doc_node: tomet::deps::DocumentNode,
        cached_proc: Option<ProcessedDoc>,
        processed_res: Result<ProcessedDoc, DocFailure>,
    }

    // Extract DocumentNode and process ProcessedDoc for modified/added files
    let processed_updates: Vec<ProcessedUpdate> = parsed_updates
        .into_par_iter()
        .map(|update| {
            let rel_path = update.rel_path;
            let (doc, _) = match update.parsed_doc {
                Ok(pair) => pair,
                Err(failure) => {
                    let dummy_node = tomet::deps::DocumentNode {
                        path: rel_path.clone(),
                        content_hash: 0,
                        exports_hash: 0,
                        outgoing: Vec::new(),
                    };
                    return ProcessedUpdate {
                        rel_path,
                        signature: update.signature,
                        doc_node: dummy_node,
                        cached_proc: None,
                        processed_res: Err(failure),
                    };
                }
            };

            let doc_node =
                tomet::deps::extract_document_node(&rel_path, &update.source, &doc, resolve_target);

            if unpublished.contains(&rel_path) {
                let dummy_failure = DocFailure {
                    rel_path: rel_path.clone(),
                    error: "unpublished document kind".to_string(),
                };
                return ProcessedUpdate {
                    rel_path,
                    signature: update.signature,
                    doc_node,
                    cached_proc: None,
                    processed_res: Err(dummy_failure),
                };
            }

            let processed_res = document::process_parsed_document_with_custom_element(
                doc,
                &rel_path,
                Some(&update.abs_path),
                config,
                &scanned.vault_index,
                &scanned.workspace_config_blocks,
                &unpublished,
                Some(&route_table),
                custom_element.clone(),
            )
            .map_err(|e| DocFailure {
                rel_path: rel_path.clone(),
                error: format!("could not process document: {e}"),
            });

            let cached_proc = processed_res.as_ref().ok().cloned();
            ProcessedUpdate {
                rel_path,
                signature: update.signature,
                doc_node,
                cached_proc,
                processed_res,
            }
        })
        .collect();

    // Partition processed updates for cache reconciliation
    let mut cache_updates = Vec::with_capacity(processed_updates.len());
    let mut newly_processed_map = HashMap::with_capacity(processed_updates.len());

    for item in processed_updates {
        cache_updates.push((
            item.rel_path.clone(),
            item.signature,
            item.doc_node,
            item.cached_proc,
        ));
        newly_processed_map.insert(item.rel_path, item.processed_res);
    }

    // Reconcile into updated cache and Change list
    let (changes, new_cache) = reconcile_vault_state(detection.clone(), cache_updates);

    // Compute affected documents
    let affected = if prev_cache.is_empty() {
        // Cold build: everything requires building
        let mut aff = AffectedPaths::default();
        for rel in new_cache.entries.keys() {
            aff.rebuild_required.insert(rel.clone());
        }
        aff
    } else {
        compute_affected_paths(prev_cache.all_nodes(), &changes)
    };

    // Assemble final complete list of processed documents in original vault scan order
    let mut processed = Vec::with_capacity(scanned.doc_files.len());
    let parsed_for_index = Vec::new();

    for doc_file in &scanned.doc_files {
        let rel = &doc_file.rel_path;
        if unpublished.contains(rel) {
            continue;
        }

        if let Some(res) = newly_processed_map.remove(rel) {
            processed.push(res);
        } else if let Some(entry) = new_cache.get(rel) {
            if let Some(p) = &entry.processed {
                processed.push(Ok(p.clone()));
            } else {
                processed.push(Err(DocFailure {
                    rel_path: rel.clone(),
                    error: "missing cached processed document".to_string(),
                }));
            }
        }
    }

    Ok(IncrementalVaultState {
        route_table,
        unpublished,
        processed,
        new_cache,
        affected,
        changes,
        parsed_for_index,
    })
}
