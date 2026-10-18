{
  craneLib,
  lib,
  tailwindcss,
  esbuild,
}:
import ./tomet-shelf.nix {
  inherit
    craneLib
    lib
    tailwindcss
    esbuild
    ;
  features = [ "embedded-search" ];
}
