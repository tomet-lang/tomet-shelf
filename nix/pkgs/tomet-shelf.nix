{
  craneLib,
  lib,
  tailwindcss_4,
  esbuild,
  lightningcss,
  # Cargo features to build with, e.g. [ "embedded-search" ]. See
  # ./tomet-shelf-embedded-search.nix for the variant that sets this.
  features ? [ ],
}:
let
  root = ../..;

  # Only what the compiler actually reads: the cargo sources, plus the CSS, JS
  # and templates under frontend/ that `include_str!` bakes into the binary,
  # (shelf.css, tailwind-input.css, and its @source globs are read by build.rs).
  # Everything else in the repo -- notes in tmtroot/, the justfile, the flake
  # files -- would otherwise trigger a full recompile when edited.
  src = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.unions [
      (craneLib.fileset.commonCargoSources root)
      (root + "/crates/tomet-shelf-assets/frontend")
      (root + "/crates/tomet-shelf-assets/templates")
      (root + "/crates/tomet-shelf-icons/assets")
    ];
  };

  commonArgs = {
    inherit src;
    pname = "tomet-shelf";
    version = "0.1.0";
    # build.rs shells out to `tailwindcss`, `lightningcss`, and `esbuild` to
    # compile utility CSS, bundle/minify stylesheets, and bundle JS; the
    # sandboxed build needs them on PATH same as `nix/dev.nix` provides them.
    nativeBuildInputs = [
      tailwindcss_4
      esbuild
      lightningcss
    ];
    cargoExtraArgs =
      "-p tomet-shelf-cli"
      + lib.optionalString (features != [ ]) " --features ${lib.concatStringsSep "," features}";
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    doCheck = false;
  }
)
