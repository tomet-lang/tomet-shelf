//! Cache entry and filesystem signature data models.

use serde::{Deserialize, Serialize};
use std::fs::Metadata;
use std::time::UNIX_EPOCH;
use tomet::deps::DocumentNode;
use tomet_shelf_document::ProcessedDoc;

/// Filesystem fingerprint of a source document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSignature {
    /// Modification time seconds since UNIX epoch.
    pub mtime_secs: u64,
    /// Subsecond nanoseconds of modification time.
    pub mtime_nanos: u32,
    /// Size of the file in bytes.
    pub file_size: u64,
}

impl FileSignature {
    /// Extracts a signature from standard filesystem [`Metadata`].
    pub fn from_metadata(meta: &Metadata) -> Self {
        let (mtime_secs, mtime_nanos) = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| (d.as_secs(), d.subsec_nanos()))
            .unwrap_or((0, 0));

        Self {
            mtime_secs,
            mtime_nanos,
            file_size: meta.len(),
        }
    }

    /// Returns true if this signature matches another (e.g. disk metadata matches cache).
    pub fn matches(&self, other: &Self) -> bool {
        self.mtime_secs == other.mtime_secs
            && self.mtime_nanos == other.mtime_nanos
            && self.file_size == other.file_size
    }
}

/// A cached record for a single document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Vault-relative path of the document (e.g. `"guides/intro.tmt"`).
    pub rel_path: String,
    /// Filesystem signature at the time the document was parsed.
    pub signature: FileSignature,
    /// Extracted document dependency node (hashes and outgoing links).
    pub doc_node: DocumentNode,
    /// Cached processed document AST & HTML rendering output, if available.
    pub processed: Option<ProcessedDoc>,
}

impl CacheEntry {
    /// Creates a new cache entry.
    pub fn new(
        rel_path: String,
        signature: FileSignature,
        doc_node: DocumentNode,
        processed: Option<ProcessedDoc>,
    ) -> Self {
        Self {
            rel_path,
            signature,
            doc_node,
            processed,
        }
    }
}
