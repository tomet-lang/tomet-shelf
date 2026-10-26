# Task: Flatten document crate and move IconRenderer to icons crate

## Goal
1. Move `IconRenderer` trait and `IconRequest` from `tomet-shelf-document` to `tomet-shelf-icons`.
2. Move `@doc.icon` HTML element rendering to `tomet-shelf-render/src/doc_icon.rs`.
3. Rename `page_icon.rs` to `icon.rs` (defining `DocIcon` and alias `PageIcon`).
4. Flatten `crates/tomet-shelf-headless-document/src/document/*` directly into `src/`.
5. Remove all icon rendering and HTML generation from `tomet-shelf-document`.
6. Update all dependent crates (`tomet-shelf-headless`, `tomet-shelf-template`, `tomet-shelf-render`).
7. Verify all tests and documentation pass.

## Steps
- [x] 1. Move `IconRenderer` & `IconRequest` into `tomet-shelf-icons`, removing `tomet-shelf-document` dependency from `tomet-shelf-icons/Cargo.toml`.
- [x] 2. Move `@doc.icon` HTML rendering into `tomet-shelf-render/src/doc_icon.rs` and wire in `tomet-shelf-render`.
- [x] 3. Flatten `tomet-shelf-headless-document/src/document/*` to `src/`, create `src/icon.rs` with `DocIcon`, and remove `src/document/`.
- [x] 4. Update `tomet-shelf-headless` to import `IconRenderer` from `tomet_shelf_icons` and use `tomet_shelf_render` for `@doc.icon` custom element rendering.
- [x] 5. Update `tomet-shelf-template` to import `IconRenderer` from `tomet_shelf_icons`.
- [x] 6. Update documentation under `tmtroot/` and run `just docs` / `just docs-check`.
- [x] 7. Run `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings`.
