use std::fs;
use std::path::PathBuf;
use tomet_shelf_incremental::{
    CACHE_FILE_NAME, CacheEntry, Dependency, DependencyKind, DocumentNode, FileSignature,
    ScannedDoc, VaultCache, compute_affected_paths, detect_changes, reconcile_vault_state,
};

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tomet-shelf-incremental-{}-{}-{:?}",
        name,
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn sample_node(
    path: &str,
    content: &str,
    exports: &str,
    outgoing: Vec<(&str, DependencyKind)>,
) -> DocumentNode {
    DocumentNode {
        path: path.to_string(),
        content_hash: tomet::deps::fnv1a_hash(content.as_bytes()),
        exports_hash: tomet::deps::fnv1a_hash(exports.as_bytes()),
        outgoing: outgoing
            .into_iter()
            .map(|(t, k)| Dependency {
                target: t.to_string(),
                kind: k,
                raw: t.to_string(),
            })
            .collect(),
    }
}

#[test]
fn test_cache_round_trip_serialization() {
    let dir = scratch_dir("roundtrip");

    let mut cache = VaultCache::new();
    let sig = FileSignature {
        mtime_secs: 1700000000,
        mtime_nanos: 123456,
        file_size: 42,
    };
    let node = sample_node("notes/alpha.tmt", "Alpha", "Title", vec![]);
    cache.insert(CacheEntry::new(
        "notes/alpha.tmt".to_string(),
        sig,
        node,
        None,
    ));

    cache.write(&dir).expect("should write cache");
    assert!(dir.join(CACHE_FILE_NAME).exists());

    let loaded = VaultCache::read(&dir).expect("should load cache");
    assert_eq!(loaded.len(), 1);

    let entry = loaded.get("notes/alpha.tmt").expect("entry exists");
    assert_eq!(entry.rel_path, "notes/alpha.tmt");
    assert_eq!(entry.signature.mtime_secs, 1700000000);
    assert_eq!(entry.signature.file_size, 42);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_detect_changes_partitions_correctly() {
    let mut cache = VaultCache::new();

    let sig_unchanged = FileSignature {
        mtime_secs: 100,
        mtime_nanos: 0,
        file_size: 50,
    };
    let sig_modified_old = FileSignature {
        mtime_secs: 100,
        mtime_nanos: 0,
        file_size: 80,
    };
    let sig_removed = FileSignature {
        mtime_secs: 100,
        mtime_nanos: 0,
        file_size: 20,
    };

    cache.insert(CacheEntry::new(
        "unchanged.tmt".to_string(),
        sig_unchanged,
        sample_node("unchanged.tmt", "Same", "Same", vec![]),
        None,
    ));
    cache.insert(CacheEntry::new(
        "modified.tmt".to_string(),
        sig_modified_old,
        sample_node("modified.tmt", "Old", "Old", vec![]),
        None,
    ));
    cache.insert(CacheEntry::new(
        "removed.tmt".to_string(),
        sig_removed,
        sample_node("removed.tmt", "Deleted", "Deleted", vec![]),
        None,
    ));

    // Current scanned files
    let scanned = vec![
        ScannedDoc {
            rel_path: "unchanged.tmt".to_string(),
            abs_path: PathBuf::from("/mock/unchanged.tmt"),
            signature: sig_unchanged,
        },
        ScannedDoc {
            rel_path: "modified.tmt".to_string(),
            abs_path: PathBuf::from("/mock/modified.tmt"),
            signature: FileSignature {
                mtime_secs: 150, // Updated timestamp!
                mtime_nanos: 0,
                file_size: 85,
            },
        },
        ScannedDoc {
            rel_path: "added.tmt".to_string(),
            abs_path: PathBuf::from("/mock/added.tmt"),
            signature: FileSignature {
                mtime_secs: 200,
                mtime_nanos: 0,
                file_size: 30,
            },
        },
    ];

    let result = detect_changes(&cache, scanned);

    assert_eq!(result.unchanged.len(), 1);
    assert_eq!(result.unchanged[0].rel_path, "unchanged.tmt");

    assert_eq!(result.modified.len(), 1);
    assert_eq!(result.modified[0].rel_path, "modified.tmt");

    assert_eq!(result.added.len(), 1);
    assert_eq!(result.added[0].rel_path, "added.tmt");

    assert_eq!(result.removed.len(), 1);
    assert_eq!(result.removed[0], "removed.tmt");
}

#[test]
fn test_reconcile_and_compute_affected_paths() {
    let mut cache = VaultCache::new();

    // doc_a links to doc_b
    let node_a = sample_node(
        "doc_a.tmt",
        "Hello [[doc_b]]",
        "A",
        vec![("doc_b.tmt", DependencyKind::Link)],
    );
    let node_b = sample_node("doc_b.tmt", "Old Title", "Old Title", vec![]);

    let sig_a = FileSignature {
        mtime_secs: 100,
        mtime_nanos: 0,
        file_size: 20,
    };
    let sig_b = FileSignature {
        mtime_secs: 100,
        mtime_nanos: 0,
        file_size: 20,
    };

    cache.insert(CacheEntry::new("doc_a.tmt".into(), sig_a, node_a, None));
    cache.insert(CacheEntry::new("doc_b.tmt".into(), sig_b, node_b, None));

    // doc_a is unchanged; doc_b modified title
    let scanned = vec![
        ScannedDoc {
            rel_path: "doc_a.tmt".into(),
            abs_path: PathBuf::from("/mock/doc_a.tmt"),
            signature: sig_a,
        },
        ScannedDoc {
            rel_path: "doc_b.tmt".into(),
            abs_path: PathBuf::from("/mock/doc_b.tmt"),
            signature: FileSignature {
                mtime_secs: 105,
                mtime_nanos: 0,
                file_size: 25,
            },
        },
    ];

    let detection = detect_changes(&cache, scanned);
    assert_eq!(detection.unchanged.len(), 1);
    assert_eq!(detection.modified.len(), 1);

    let new_node_b = sample_node("doc_b.tmt", "New Title", "New Title", vec![]);
    let parsed_updates = vec![(
        "doc_b.tmt".to_string(),
        detection.modified[0].signature,
        new_node_b,
        None,
    )];

    let (changes, updated_cache) = reconcile_vault_state(detection, parsed_updates);
    assert_eq!(changes.len(), 1);
    assert_eq!(updated_cache.len(), 2);

    // Compute affected paths against the pre-update baseline graph
    let affected = compute_affected_paths(cache.all_nodes(), &changes);

    // doc_b exports changed, so doc_a (which references doc_b) and doc_b must both be rebuilt!
    assert!(affected.rebuild_required.contains("doc_a.tmt"));
    assert!(affected.rebuild_required.contains("doc_b.tmt"));
}

#[test]
fn test_content_only_change_does_not_propagate_to_link_referrers() {
    let mut cache = VaultCache::new();

    let node_a = sample_node(
        "doc_a.tmt",
        "Links to B",
        "Title A",
        vec![("doc_b.tmt", DependencyKind::Link)],
    );
    let node_b = sample_node("doc_b.tmt", "Original Body", "Title B", vec![]);

    let sig_a = FileSignature {
        mtime_secs: 100,
        mtime_nanos: 0,
        file_size: 20,
    };
    let sig_b = FileSignature {
        mtime_secs: 100,
        mtime_nanos: 0,
        file_size: 20,
    };

    cache.insert(CacheEntry::new("doc_a.tmt".into(), sig_a, node_a, None));
    cache.insert(CacheEntry::new("doc_b.tmt".into(), sig_b, node_b, None));

    let scanned = vec![
        ScannedDoc {
            rel_path: "doc_a.tmt".into(),
            abs_path: PathBuf::from("/mock/doc_a.tmt"),
            signature: sig_a,
        },
        ScannedDoc {
            rel_path: "doc_b.tmt".into(),
            abs_path: PathBuf::from("/mock/doc_b.tmt"),
            signature: FileSignature {
                mtime_secs: 105,
                mtime_nanos: 0,
                file_size: 25,
            },
        },
    ];

    let detection = detect_changes(&cache, scanned);
    // doc_b modified content, but exports (Title B) and outgoing are identical!
    let new_node_b = sample_node("doc_b.tmt", "Updated Body Text", "Title B", vec![]);
    let parsed_updates = vec![(
        "doc_b.tmt".to_string(),
        detection.modified[0].signature,
        new_node_b,
        None,
    )];

    let (changes, _) = reconcile_vault_state(detection, parsed_updates);
    let affected = compute_affected_paths(cache.all_nodes(), &changes);

    assert!(affected.rebuild_required.contains("doc_b.tmt"));
    assert!(
        !affected.rebuild_required.contains("doc_a.tmt"),
        "doc_a should NOT be rebuilt on text-only edit of doc_b"
    );
}

#[test]
fn test_transclusion_embed_propagates_content_change_to_parent() {
    let mut cache = VaultCache::new();

    // doc_a EMBEDS doc_b
    let node_a = sample_node(
        "doc_a.tmt",
        "Embeds B",
        "Title A",
        vec![("doc_b.tmt", DependencyKind::Embed)],
    );
    let node_b = sample_node("doc_b.tmt", "Snippet v1", "Title B", vec![]);

    let sig_a = FileSignature {
        mtime_secs: 100,
        mtime_nanos: 0,
        file_size: 20,
    };
    let sig_b = FileSignature {
        mtime_secs: 100,
        mtime_nanos: 0,
        file_size: 20,
    };

    cache.insert(CacheEntry::new("doc_a.tmt".into(), sig_a, node_a, None));
    cache.insert(CacheEntry::new("doc_b.tmt".into(), sig_b, node_b, None));

    let scanned = vec![
        ScannedDoc {
            rel_path: "doc_a.tmt".into(),
            abs_path: PathBuf::from("/mock/doc_a.tmt"),
            signature: sig_a,
        },
        ScannedDoc {
            rel_path: "doc_b.tmt".into(),
            abs_path: PathBuf::from("/mock/doc_b.tmt"),
            signature: FileSignature {
                mtime_secs: 105,
                mtime_nanos: 0,
                file_size: 25,
            },
        },
    ];

    let detection = detect_changes(&cache, scanned);
    // doc_b modified text only
    let new_node_b = sample_node("doc_b.tmt", "Snippet v2", "Title B", vec![]);
    let parsed_updates = vec![(
        "doc_b.tmt".to_string(),
        detection.modified[0].signature,
        new_node_b,
        None,
    )];

    let (changes, _) = reconcile_vault_state(detection, parsed_updates);
    let affected = compute_affected_paths(cache.all_nodes(), &changes);

    assert!(affected.rebuild_required.contains("doc_b.tmt"));
    assert!(
        affected.rebuild_required.contains("doc_a.tmt"),
        "doc_a MUST be rebuilt because it embeds doc_b"
    );
}
