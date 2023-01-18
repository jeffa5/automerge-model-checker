{
  description = "Automerge Model Checking";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem
    (system: let
      pkgs = import nixpkgs {
        overlays = [rust-overlay.overlays.default];
        inherit system;
      };
      lib = pkgs.lib;
      rust = pkgs.rust-bin.stable.latest.default;
      cargoNix = pkgs.callPackage ./Cargo.nix {
        inherit pkgs;
        release = true;
      };
      debugCargoNix = pkgs.callPackage ./Cargo.nix {
        inherit pkgs;
        release = false;
      };
    in {
      formatter = pkgs.alejandra;

      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          (rust.override {
            extensions = ["rust-src"];
          })
          cargo-edit
          cargo-watch
          cargo-criterion
          cargo-fuzz
          cargo-flamegraph
          cargo-deny
          crate2nix

          rnix-lsp
          nixpkgs-fmt
        ];
      };
    });
}
