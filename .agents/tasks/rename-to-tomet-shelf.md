# Rename tomet-book / tmtbook to tomet-shelf

Decided with the user (2026-10-18):

- One name per product, language name unabbreviated, hyphen: `tomet-shelf`
  (like `rust-analyzer`; family: `tomet-lsp`, `tomet-shelf`).
- Repository, binary, crate and flake attribute are all `tomet-shelf`.
  Library crates: `tomet-shelf-assets`, `-config`, `-render`, `-search`, `-serve`.
- Config file: `shelf.toml` (the noun, as mdbook has `book.toml`).
- Identifiers inside generated pages (storage keys, console prefix): `shelf-` / `[shelf]`.
- No backward compatibility: no old config name, no migration of old storage keys.
- GitHub repository already renamed to `tomet-lang/tomet-shelf`; local `origin` updated.
- Pages `base`: `/tomet-shelf`.

Done and reported only at the end of each step; nothing is committed without the user's ask.

## Steps

- [x] 1. Workspace `.agents/workspaces/rename-shelf` created.
- [x] 2. Crate directories and `Cargo.toml` names (workspace, app, six crates), `use`/path references.
- [x] 3. Binary name in `app/src/cli.rs`, config file name `shelf.toml` in code and in `tmtroot/`.
- [x] 4. Frontend identifiers (`shelf-*` storage keys, `[shelf]` log prefix), emitted assets
      `shelf.css` / `shelf.js`, route `/__shelf/source`.
- [x] 5. Nix files, `justfile`, CI rewritten (not yet built with nix; only `cargo check` done).
- [x] `cargo check --all-targets` passes in the workspace; `Cargo.lock` differs only by the renames.
- [x] 6. Documentation sources under `tmtroot/` (`what-is-tomet-shelf.tmt` renamed too), `just docs` and `just docs-check` pass.
- [x] 7. `cargo update --workspace` (lock differs only by the renames), `cargo test --workspace` all green,
      `nix build .#tomet-shelf` builds and `tomet-shelf --version` prints `tomet-shelf 0.1.0`.
      Not run: `just ci` as a whole (clippy, nix fmt), regenerated `dist/` (untracked).
- [x] 7b. Committed and pushed as `4c8811bf` (main); workspace forgotten and removed; the local
      directory is now `tomet-shelf` (was `tomet-book`).
- [ ] 8. Consumers, one by one, each in its own commit. Every vault that has a `tmtroot/tmtbook.toml`
      must also rename it to `shelf.toml` (found only by file name, not by content): tomet,
      tomet-corpus, tomet-web-editor, immermemo, tomet-website, twrit, examples/app-dev,
      tomet-corpus/data/examples/app-dev.
  - [x] Committed and pushed (2026-10-18), each as "follow the rename of tomet-book to tomet-shelf":
        tomet-corpus (`cargo check` passes), tomet, twrit, immermemo (these three by explicit paths,
        the user's own edits stay in the working copy), tomet-web-editor, tomet-website, examples, sisho.
  - [ ] oneline: lines rewritten, NOT committed (all its files are the user's new uncommitted work).
        Its `nix/default.nix:22` has `inherit tomet;` with no `tomet` in scope (not from this task).
  - [ ] arbor-florifera: lines rewritten, NOT relocked (not a git repository; `nix flake lock` on a
        path flake would copy `target/`). Run `nix flake lock` after `git init`/committing it.
  - Lock growth: relocking tomet/twrit/immermemo added ~146 nodes each, because the pinned
    tomet/twrit revisions still depend on `tomet-book`; the old branches disappear once tomet and
    twrit themselves are re-pinned on the renamed revisions. The sisho `toolchain` bundle is the fix.
  - Not done: regenerated site output (`dist/`, `tmtroot/*.html`, untracked everywhere);
    `tomet-corpus/data/examples` (untracked sync copy) still has `tmtbook.toml`.
- [ ] 9. Remove this file after folding anything worth keeping into the design memo of sisho.

## Inventory (before the work)

About 90 hand-written files, ~350 generated HTML files that are regenerated.
tomet-book itself ~70 files; tomet-corpus is the heaviest consumer
(`tmtbook::BookConfig::load_from_dir`, `tmtbook::build_book`).
