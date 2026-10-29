//! Persistent vault cache file management and serialization.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

use crate::entry::CacheEntry;

/// Filename of the on-disk incremental cache.
pub const CACHE_FILE_NAME: &str = ".tomet-shelf-cache.json";

/// Current schema version of the cache.
/// Incrementing this invalidates older cache files safely.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// In-memory and persistent container for cached document entries across a vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultCache {
    /// Schema version for compatibility checking.
    pub schema_version: u32,
    /// Cached entries keyed by vault-relative document path.
    pub entries: HashMap<String, CacheEntry>,
}

impl Default for VaultCache {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            entries: HashMap::new(),
        }
    }
}

impl VaultCache {
    /// Creates an empty cache instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the cache contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Gets a reference to an entry by relative path.
    pub fn get(&self, rel_path: &str) -> Option<&CacheEntry> {
        self.entries.get(rel_path)
    }

    /// Iterator over all document dependency nodes currently in the cache.
    pub fn all_nodes(&self) -> impl Iterator<Item = tomet::deps::DocumentNode> + '_ {
        self.entries.values().map(|e| e.doc_node.clone())
    }

    /// Inserts or replaces a cache entry.
    pub fn insert(&mut self, entry: CacheEntry) {
        self.entries.insert(entry.rel_path.clone(), entry);
    }

    /// Removes an entry by relative path.
    pub fn remove(&mut self, rel_path: &str) -> Option<CacheEntry> {
        self.entries.remove(rel_path)
    }

    /// Reads and deserializes the cache from the specified directory (e.g. output dir).
    /// Returns `None` if the file does not exist, is invalid JSON, or has an outdated schema.
    pub fn read(dir: &Path) -> Option<Self> {
        let path = dir.join(CACHE_FILE_NAME);
        if !path.exists() {
            debug!("Incremental cache not found at {}", path.display());
            return None;
        }

        let raw = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                warn!("Failed to read cache file at {}: {e}", path.display());
                return None;
            }
        };

        match serde_json::from_str::<VaultCache>(&raw) {
            Ok(cache) => {
                if cache.schema_version != CURRENT_SCHEMA_VERSION {
                    debug!(
                        "Ignoring cache at {} due to schema version mismatch (expected {}, found {})",
                        path.display(),
                        CURRENT_SCHEMA_VERSION,
                        cache.schema_version
                    );
                    None
                } else {
                    debug!(
                        "Loaded incremental cache with {} entries from {}",
                        cache.entries.len(),
                        path.display()
                    );
                    Some(cache)
                }
            }
            Err(e) => {
                warn!("Failed to parse cache at {}: {e}", path.display());
                None
            }
        }
    }

    /// Atomically writes the cache to the specified directory.
    pub fn write(&self, dir: &Path) -> Result<()> {
        let path = dir.join(CACHE_FILE_NAME);
        let temp_path = dir.join(format!("{CACHE_FILE_NAME}.tmp.{}", std::process::id()));

        let json =
            serde_json::to_string(self).context("Failed to serialize incremental cache to JSON")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        fs::write(&temp_path, json.as_bytes()).with_context(|| {
            format!("Failed to write temporary cache at {}", temp_path.display())
        })?;

        fs::rename(&temp_path, &path).with_context(|| {
            format!(
                "Failed to atomically rename {} to {}",
                temp_path.display(),
                path.display()
            )
        })?;

        debug!(
            "Saved incremental cache ({} entries) to {}",
            self.entries.len(),
            path.display()
        );
        Ok(())
    }
}
