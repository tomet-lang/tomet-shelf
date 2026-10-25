# Task: Create `stela-shelf` Repository

## Context
- `stela-shelf` (`kakusika/stela-shelf`) is the stone/monument-inspired custom shelf variant built on top of `tomet-shelf`'s headless engine.
- `tomet-shelf` (`tomet-lang/tomet-shelf`) remains the official SSG/documentation server (aiming for an Obsidian × Wikipedia feature-rich, clean standard UI) and hosts the shared headless crates.
- User confirmed:
  1. Create local repository at `/home/tefla/projects/develop/project-tomet/stela-shelf`.
  2. Remote `kakusika/stela-shelf` is already registered on GitHub.
  3. Do NOT finalize commits or push yet — stop once the working tree is prepared so the user can review.

## Steps
- [x] Inspect `tomet-shelf` root & `tmtroot/` structure to match project conventions (`catasto.ctmt`, `flake.nix`, `nix/`, `justfile`, `tmtroot/`, `.gitignore`, `CLAUDE.md`).
- [x] Create `/home/tefla/projects/develop/project-tomet/stela-shelf` and initialize colocated Jujutsu repository (`jj git init --colocate`) with remote `origin` pointing to `kakusika/stela-shelf`.
- [x] Set up `catasto.ctmt` in `stela-shelf` and sync upstream `commonfiles` via `catasto`.
- [x] Scaffold initial `stela-shelf` workspace files (`flake.nix`, `nix/`, `justfile`, `.gitignore`, `tmtroot/`, `CLAUDE.md`) and generate derived markdown docs (`sisho fix .` / `tomet export .`).
- [x] Verify `stela-shelf` status and present the prepared repository to the user for review (without committing).
