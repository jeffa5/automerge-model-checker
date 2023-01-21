{
  description = "Automerge Model Checking";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  nixConfig = {
    extra-substituters = [
      "https://automerge-model-checking.cachix.org"
    ];
    extra-trusted-public-keys = ["automerge-model-checking.cachix.org-1:le7f4sh93Kr0n1F8/5AjyEe883EXQxkrUKVVZDHmMiY="];
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      overlays = [rust-overlay.overlays.default];
      inherit system;
    };
    craneLib = crane.lib.${system};
    rust = pkgs.rust-bin.stable.latest.default;
    src = craneLib.cleanCargoSource ./.;
    cargoArtifacts = craneLib.buildDepsOnly {
      inherit src;
    };
  in {
    packages.${system} = {
      default = self.packages.${system}.amc;

      amc = craneLib.buildPackage {
        inherit cargoArtifacts src;
      };
    };

    formatter.${system} = pkgs.alejandra;

    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        (rust.override {
          extensions = ["rust-src"];
        })
        cargo-flamegraph
      ];
    };
  };
}
