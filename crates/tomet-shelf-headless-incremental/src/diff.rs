//! Change detection between filesystem state and the incremental cache.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tomet::deps::{Change, DocumentNode};
use tomet_shelf_document::ProcessedDoc;

use crate::cache::VaultCache;
use crate::entry::{CacheEntry, FileSignature};

/// Descriptor of a scanned document file on disk.
#[derive(Debug, Clone)]
pub struct ScannedDoc {
    /// Relative path within the vault (e.g. `"notes/alpha.tmt"`).
    pub rel_path: String,
    /// Absolute path on disk.
    pub abs_path: PathBuf,
    /// Filesystem modification signature.
    pub signature: FileSignature,
}

impl ScannedDoc {
    /// Scans a file on disk, computing its [`FileSignature`].
    pub fn from_path(base_dir: &Path, rel_path: String) -> std::io::Result<Self> {
        let abs_path = base_dir.join(&rel_path);
        let meta = std::fs::metadata(&abs_path)?;
        let signature = FileSignature::from_metadata(&meta);
        Ok(Self {
            rel_path,
            abs_path,
            signature,
        })
    }
}

/// A document that changed on disk and requires re-reading and parsing.
#[derive(Debug, Clone)]
pub struct ChangedDoc {
    pub rel_path: String,
    pub abs_path: PathBuf,
    pub signature: FileSignature,
    pub prev_entry: Option<CacheEntry>,
}

/// Partition of scanned files compared against the previous cache.
#[derive(Debug, Clone)]
pub struct ChangeDetectionResult {
    /// Cached entries whose file signature matched disk exactly (zero I/O needed).
    pub unchanged: Vec<CacheEntry>,
    /// Documents modified since the last build.
    pub modified: Vec<ChangedDoc>,
    /// Newly added documents not present in the cache.
    pub added: Vec<ChangedDoc>,
    /// Documents removed from the vault.
    pub removed: Vec<String>,
}

impl ChangeDetectionResult {
    /// Returns true if no files were added, modified, or removed.
    pub fn is_clean(&self) -> bool {
        self.modified.is_empty() && self.added.is_empty() && self.removed.is_empty()
    }
}

/// Detects changes by comparing current scanned vault files against the previous cache.
pub fn detect_changes(
    cache: &VaultCache,
    scanned: impl IntoIterator<Item = ScannedDoc>,
) -> ChangeDetectionResult {
    let mut scanned_map: HashMap<String, ScannedDoc> = HashMap::new();
    for doc in scanned {
        scanned_map.insert(doc.rel_path.clone(), doc);
    }

    let mut unchanged = Vec::new();
    let mut modified = Vec::new();
    let mut added = Vec::new();
    let mut seen_in_scanned = HashSet::new();

    for (rel_path, scanned_doc) in scanned_map {
        seen_in_scanned.insert(rel_path.clone());
        if let Some(cached_entry) = cache.get(&rel_path) {
            if cached_entry.signature.matches(&scanned_doc.signature) {
                unchanged.push(cached_entry.clone());
            } else {
                modified.push(ChangedDoc {
                    rel_path,
                    abs_path: scanned_doc.abs_path,
                    signature: scanned_doc.signature,
                    prev_entry: Some(cached_entry.clone()),
                });
            }
        } else {
            added.push(ChangedDoc {
                rel_path,
                abs_path: scanned_doc.abs_path,
                signature: scanned_doc.signature,
                prev_entry: None,
            });
        }
    }

    // Detect removed files: present in cache but absent from current scanned files
    let mut removed = Vec::new();
    for cached_path in cache.entries.keys() {
        if !seen_in_scanned.contains(cached_path) {
            removed.push(cached_path.clone());
        }
    }

    ChangeDetectionResult {
        unchanged,
        modified,
        added,
        removed,
    }
}

/// Helper to reconcile unchanged entries with newly parsed updates into updated cache and change list.
pub fn reconcile_vault_state(
    detection: ChangeDetectionResult,
    parsed_updates: Vec<(String, FileSignature, DocumentNode, Option<ProcessedDoc>)>,
) -> (Vec<Change>, VaultCache) {
    let mut changes = Vec::new();
    let mut new_cache = VaultCache::new();

    // Preserve unchanged entries
    for entry in detection.unchanged {
        new_cache.insert(entry);
    }

    // Process removed files
    for rem in detection.removed {
        changes.push(Change::Removed(rem));
    }

    // Process parsed modified/added files
    let mut updates_map: HashMap<String, (FileSignature, DocumentNode, Option<ProcessedDoc>)> =
        parsed_updates
            .into_iter()
            .map(|(p, sig, node, proc)| (p, (sig, node, proc)))
            .collect();

    for mod_doc in detection.modified {
        if let Some((sig, node, proc)) = updates_map.remove(&mod_doc.rel_path) {
            changes.push(Change::Modified(node.clone()));
            new_cache.insert(CacheEntry::new(mod_doc.rel_path, sig, node, proc));
        }
    }

    for add_doc in detection.added {
        if let Some((sig, node, proc)) = updates_map.remove(&add_doc.rel_path) {
            changes.push(Change::Added(node.clone()));
            new_cache.insert(CacheEntry::new(add_doc.rel_path, sig, node, proc));
        }
    }

    (changes, new_cache)
}
