{
  flake-parts,
  ...
}@inputs:
flake-parts.lib.mkFlake { inherit inputs; } {
  systems = [
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
  ];
  imports = [
    inputs.treefmt-nix.flakeModule
  ];

  perSystem =
    { pkgs, ... }:
    let
      system = pkgs.stdenv.hostPlatform.system;
      craneLib = inputs.crane.mkLib pkgs;
      tometPkgs = inputs.tomet.packages.${system};
      twritPkgs = inputs.twrit.packages.${system};
      catastoPkgs = inputs.catasto.packages.${system};
      sishoPkgs = inputs.sisho.packages.${system};
    in
    {
      packages = rec {
        default = tomet-shelf;
        tomet-shelf = pkgs.callPackage ./pkgs/tomet-shelf.nix { inherit craneLib; };
        tomet-shelf-embedded-search = pkgs.callPackage ./pkgs/tomet-shelf-embedded-search.nix {
          inherit craneLib;
        };

        # What CI builds and sends to Cachix (.github/workflows/nix-ci.yml).
        ci = pkgs.linkFarm "ci" [
          {
            name = "tomet-shelf";
            path = tomet-shelf;
          }
          {
            name = "tomet-shelf-embedded-search";
            path = tomet-shelf-embedded-search;
          }
        ];
      };

      devShells.default = pkgs.callPackage ./dev.nix {
        inherit inputs;
        fenix = inputs.fenix.packages.${system};
        tomet-shelf = pkgs.callPackage ./pkgs/tomet-shelf.nix { inherit craneLib; };
        # The other tools; tomet-shelf itself is the one built from this checkout.
        tools = pkgs.symlinkJoin {
          name = "tomet-toolchain";
          paths = [
            sishoPkgs.sisho
            tometPkgs.tomet
            tometPkgs.tomet-lsp
            twritPkgs.twrit
            # `sisho check` runs `catasto check` beside `tomet` and `twrit`.
            catastoPkgs.catasto
          ];
        };
      };

      treefmt = import ./formatter.nix {
        inherit (tometPkgs) tomet;
      };
    };
}
