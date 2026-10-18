{
  description = "Tomet Book generator (tomet-shelf) - A standalone book & wiki generator for Tomet";

  nixConfig = {
    extra-substituters = [ "https://tomet.cachix.org" ];
    extra-trusted-public-keys = [ "tomet.cachix.org-1:9c/iO8Tb6YOM+3r55t12W3fOJK+66itPEaIe8Rs9MDw=" ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    #= Rust
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };

    #= Tool
    # sisho brings tomet and twrit for the dev shell. It pins tomet-shelf too, but its
    # `tomet-shelf` input cuts this one (`inputs.sisho.follows = ""`), so the loop is not in the lock.
    sisho.url = "github:tomet-lang/sisho";
  };

  outputs = inputs: import ./nix inputs;
}
