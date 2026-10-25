# Checks that were red on `main` (found while taking commonfiles, 2026-10-24)

- [x] CI "Run CI checks & tests" (`just ci`) failed on the last five runs of `main`: `nix fmt -- --ci` wanted four Rust files
  (`tomet-shelf-assets/src/lib.rs`, `tomet-shelf-render/src/document/code.rs`, `tomet-shelf/src/book/image_opt.rs`, `tomet-shelf/src/serve.rs`)
  and `nix/default.nix` (statix: `inherit (sisho) tomet;`) formatted. Done in a `style:` commit with the common files; `just check`, clippy
  `-D warnings` and `cargo test --workspace` pass.
- [ ] `tomet check .` (tomet 0.5.0) fails, so "Check docs" (`just docs-check`, `sisho check .`) stays red: 1 of 5 steps, on main before and after
  the common files. The documents still use the old heads.
- [ ] The port of the tomet-shelf documents/engine to the new tomet evaluator: `follow-tomet-main.md`, `rename-to-tomet-shelf.md`.
