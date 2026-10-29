<!-- Generated from tmtroot/readme.tmt. Edit that, then `tomet export .`. -->

# Tomet Shelf

Fast static site generator and documentation server specifically built for Tomet (`.tmt`) vaults.

## Overview

`tomet-shelf` provides a modern documentation site experience for Tomet vaults. It features:

- **Blazing fast builds**: Written in Rust, optimized for large-scale knowledge bases.
- **Instant link resolution**: First-class support for Tomet intra-vault link resolution and references.
- **Client-side search**: Full-text client-side indexing and instant search interface.
- **Live reloading development server**: Fast feedback cycle during documentation authoring.

## Workspace Layout

- `app` (`tomet-shelf-cli`) -- CLI binary (`tomet-shelf`).
- `crates/tomet-shelf-assets` (`tomet-shelf-assets`) -- Official frontend web assets, Askama templates, theme styling, client scripts, and i18n dictionaries.
- `crates/tomet-shelf-headless` (`tomet-shelf-headless`) -- Headless SSG engine, vault state, and dev server orchestration.
- `crates/tomet-shelf-headless-config` (`tomet-shelf-config`) -- `shelf.toml` configuration parser and models.
- `crates/tomet-shelf-headless-document` (`tomet-shelf-document`) -- Tomet document parsing, AST transformation, and metadata extraction.
- `crates/tomet-shelf-headless-incremental` (`tomet-shelf-incremental`) -- Incremental state, cache, and change detection.
- `crates/tomet-shelf-headless-route` (`tomet-shelf-route`) -- Route resolution, slug generation, and URL sanitization.
- `crates/tomet-shelf-headless-search` (`tomet-shelf-search`) -- Full-text search indexing.
- `crates/tomet-shelf-headless-serve` (`tomet-shelf-serve`) -- Axum development server and WebSocket live reload.

## Building and Running

### Build workspace

```bash
cargo build --workspace
```

### Run tests

```bash
cargo test --workspace
```

### Serve a vault locally

```bash
cargo run -p tomet-shelf-cli -- serve tmtroot/
```

