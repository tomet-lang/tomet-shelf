# Task: Create tomet-shelf-render and migrate HTML tag injection from headless-document

## Goal
Create `crates/tomet-shelf-render` as a dedicated HTML tag/component injection crate, and remove hardcoded presentation HTML tag generation from `crates/tomet-shelf-headless-document`.

## Steps
- [x] 1. Create `crates/tomet-shelf-render` with `Cargo.toml` and workspace registration
- [x] 2. Move HTML enhancement modules (`marker.rs`, `code.rs`, `external_links.rs`, `internal_links.rs`, avatar/chip HTML formatters) from `headless-document` to `tomet-shelf-render`
- [x] 3. Purify `headless-document`: replace HTML string generation in `meta/icon.rs`, `meta/format.rs`, and `document/mod.rs` with structured data (`PageIcon`, pure values)
- [x] 4. Wire `tomet-shelf-render` into `tomet-shelf-template` and build pipeline to apply HTML enhancements during rendering
- [x] 5. Update docs under `tmtroot/` and run `just docs` / `just docs-check`
- [x] 6. Verify compilation and test suite (`cargo check`, `cargo test`)
