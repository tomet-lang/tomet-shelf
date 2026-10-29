//! =[ tomet-shelf-incremental ]
//!
//! Incremental state, cache, and change detection for `tomet-shelf`.
//!
//! Orchestrates on-disk cache persistence, mtime/size comparison for zero-I/O
//! unchanged skips, and integration with `tomet::deps` for invalidation impact analysis.

pub mod cache;
pub mod diff;
pub mod entry;

pub use cache::{CACHE_FILE_NAME, CURRENT_SCHEMA_VERSION, VaultCache};
pub use diff::{
    ChangeDetectionResult, ChangedDoc, ScannedDoc, detect_changes, reconcile_vault_state,
};
pub use entry::{CacheEntry, FileSignature};

// Re-export core dependency tracking types from tomet::deps for convenience
pub use tomet::deps::{
    AffectedPaths, Backlink, Change, Dependency, DependencyGraph, DependencyKind, DocumentNode,
    compute_affected_paths,
};
