# Task: Rename tomet-shelf-render to tomet-shelf-document

## Goal
Rename `crates/tomet-shelf-headless-render` (package `tomet-shelf-render`) to `crates/tomet-shelf-document` (package `tomet-shelf-document`), decoupling document processing from rendering.

## Steps
- [x] 1. Rename directory `crates/tomet-shelf-headless-render` to `crates/tomet-shelf-headless-document`
- [x] 2. Update `crates/tomet-shelf-headless-document/Cargo.toml` package name to `tomet-shelf-document`
- [x] 3. Update root `Cargo.toml` workspace dependency to `tomet-shelf-document`
- [x] 4. Update dependent crates (`tomet-shelf-headless`, `tomet-shelf-assets`, `tomet-shelf-template`) Cargo.toml and Rust imports
- [x] 5. Update docs under `tmtroot/` and run `just docs`
- [x] 6. Verify compilation and test suite (`cargo check`, `cargo test`)
