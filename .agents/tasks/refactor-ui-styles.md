# Task: tomet-shelf Style Refactoring (Border-less & Shadow Depth)

## Context
- Refactor `tomet-shelf-assets/frontend/styles/` to implement the "Obsidian × Wikipedia" design spec.
- Remove stone/marble/seigaiha legacy textures and shell patterns.
- Enforce "Border-less by default" with depth and elevation created consistently through shadows and surface tones.

## Steps
- [x] 1. Create Jujutsu workspace `.agents/workspaces/refactor-ui-styles`.
- [x] 2. Update `00-base.css` with semantic color tokens, elevation shadows, and remove legacy stone patterns.
- [x] 3. Update `10-shell.css`, `20-panes.css`, and `30-sticky-tabs.css` to remove borders and apply surface tones/shadows.
- [x] 4. Update `40-hero.css`, `50-nav-infobox.css`, `60-content.css`, and `72-center-peek.css` to use border-less shadow cards.
- [x] 5. Update Web Component styles in `components/tmt-*.css` and `tailwind.config.ts`.
- [x] 6. Verify with `tsc --noEmit`, `cargo test --workspace`, and site generation (`tomet-shelf build tmtroot`).
- [x] 7. Merge and clean up workspace.
