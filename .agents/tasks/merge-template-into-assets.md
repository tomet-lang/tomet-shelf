# Task: Merge tomet-shelf-template into tomet-shelf-assets and remove template crate

## Overview
Fully merge `crates/tomet-shelf-template` (Askama renderer and HTML templates) into `crates/tomet-shelf-assets`, eliminating `tomet-shelf-template` completely.

## Steps
- [x] Step 1: Create dedicated Jujutsu workspace `.agents/workspaces/merge-template-into-assets`
- [x] Step 2: Investigate all usages of `tomet-shelf-template` across workspace and downstream
- [x] Step 3: Move `templates/` and renderer implementation into `tomet-shelf-assets`
- [x] Step 4: Update `tomet-shelf-assets/Cargo.toml` with Askama and renderer dependencies
- [x] Step 5: Delete `crates/tomet-shelf-template` and remove from workspace `Cargo.toml`
- [x] Step 6: Update all references to `tomet-shelf-template` across workspace
- [x] Step 7: Update documentation in `tmtroot/` and regenerate with `just docs`
- [x] Step 8: Verify full test suite (`cargo test`, `just check`, `just ci`)
- [ ] Step 9: Clean up workspace and prepare commit proposal
