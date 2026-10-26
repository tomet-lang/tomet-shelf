# Task: Remove All Backward Compatibility Aliases and Deprecated Shims

## Goal
Eliminate all backward compatibility shims, aliases, and deprecated artifacts across the codebase. Commit to pure, direct types and APIs without legacy baggage:
1. Eliminate `PageIcon` alias in favor of `DocIcon` across all crates.
2. Eliminate `process_parsed_document_with_renderer` alias in favor of `process_parsed_document_with_custom_element`.
3. Eliminate deprecated empty `ICON_ASSETS` from `tomet-shelf-headless` and remove its dead loop in `serve.rs`.
4. Verify workspace tests and docs.

## Steps
- [x] 1. Remove `PageIcon` alias from `tomet-shelf-document` and update all usages to `DocIcon`.
- [x] 2. Remove `process_parsed_document_with_renderer` alias from `tomet-shelf-document` and update callers to `process_parsed_document_with_custom_element`.
- [x] 3. Remove deprecated `ICON_ASSETS` from `tomet-shelf-headless` and remove dead loop in `serve.rs`.
- [x] 4. Update documentation in `tmtroot/` and run `just docs` / `just docs-check`.
- [x] 5. Verify all tests pass (`cargo check --workspace`, `cargo test --workspace`, `cargo clippy`).
