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
      sisho = inputs.sisho.packages.${system};
    in
    {
      packages = rec {
        default = tomet-shelf;
        tomet-shelf = pkgs.callPackage ./pkgs/tomet-shelf.nix { inherit craneLib; };
        tomet-shelf-embedded-search = pkgs.callPackage ./pkgs/tomet-shelf-embedded-search.nix {
          inherit craneLib;
        };
      };

      devShells.default = pkgs.callPackage ./dev.nix {
        inherit inputs;
        fenix = inputs.fenix.packages.${system};
        tomet-shelf = pkgs.callPackage ./pkgs/tomet-shelf.nix { inherit craneLib; };
        # The other tools; tomet-shelf itself is the one built from this checkout.
        tools = inputs.sisho.legacyPackages.${system}.combine [
          sisho.sisho
          sisho.tomet
          sisho.tomet-lsp
          sisho.twrit
        ];
      };

      treefmt = import ./formatter.nix {
        inherit (sisho) tomet;
      };
    };
}
