# Follow tomet `main`

tomet-shelf still pins tomet `f3dedb2c`. Bumping it does not compile: `Cargo.toml` asks for the crate `tomet-compute`,
which tomet renamed to `tomet-expr` and then replaced by `tomet-semantics-expr` with a typed evaluator
(`ac46fc05`, `b328eb33`, `e8af3f13`).

The one call site is `crates/tomet-shelf-render/src/document/mod.rs` (`process_document`):

```rust
let eval_vars = tomet_compute::EvaluationContext::new().with_var("self", Map { path, filename });
let unresolved = tomet_transform::resolve_interpolations(&mut doc, &doc_cfg, &eval_vars);
```

`tomet_transform::resolve_interpolations` is gone. The evaluator is `tomet_semantics_expr::EvalContext { phase, resolver,
generate, self_doc, macros, vars, .. }` with `Resolver`, `Notes` and `PreparedBlocks`. How tomet itself resolves
interpolations in `tomet html` is the model to follow.

- [ ] 1. Read how `tomet html` / `tomet export` prepare a document's interpolations (`tomet-semantics-expr`, `tomet-transform`) and do the same for a page.
- [ ] 2. Replace the `tomet-compute` dependency; keep the `self.path` and `self.filename` variables.
- [ ] 3. Bump `Cargo.lock` to the current tomet (cotomet rename: `.mtm` -> `.ctmt`, `motem` -> `cotomet`; tomet-shelf uses neither directly).
- [ ] 4. The documents open with `@kind`, `@meta`, `@config`, `@blueprint` (9 files); they are `::name` now.
- [ ] 5. `cargo test --workspace`, `just docs-check` in `nix develop`.

The default workspace holds uncommitted work (`rename-to-tomet-shelf.md`, `nix/dev.nix`, `tmtroot/spec/config.tmt`); do this in a workspace off `main`.
