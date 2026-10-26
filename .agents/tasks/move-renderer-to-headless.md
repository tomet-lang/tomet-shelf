# Task: Move PageRenderer and MinimalRenderer from document to headless

## Goal
Move `PageRenderer`, `RendererFactory`, `Backlink`, `EntrySummary`, `SectionSummary`, `MinimalRenderer`, and `MinimalRendererFactory` from `crates/tomet-shelf-headless-document` into `crates/tomet-shelf-headless`. Purify `tomet-shelf-document` so it only deals with document parsing, AST, metadata extraction, links, and TOC.

## Steps
- [x] 1. Move `renderer.rs` from `crates/tomet-shelf-headless-document/src/` to `crates/tomet-shelf-headless/src/renderer.rs`
- [x] 2. Also move static asset writing (`assets.rs`: `StaticAsset`, `write_static_assets`, `write_if_changed`, `content_version`) to `crates/tomet-shelf-headless/src/assets.rs`
- [x] 3. Update `crates/tomet-shelf-headless/src/lib.rs` and `book/mod.rs` to expose and use the moved types directly
- [x] 4. Remove `renderer.rs` and `assets.rs` from `tomet-shelf-document` (`crates/tomet-shelf-headless-document`)
- [x] 5. Update dependent crates (`tomet-shelf-template`, `tomet-shelf-assets`, `stela-shelf`) to import `PageRenderer` / `Backlink` / `StaticAsset` from `tomet-shelf-headless`
- [x] 6. Update `tmtroot/` documentation and run `just docs` / `just docs-check`
- [x] 7. Verify workspace builds and tests (`cargo check --workspace`, `cargo test --workspace`, `cargo clippy`)
