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
    pname = "amc";
    cargoArtifacts = craneLib.buildDepsOnly {
      inherit src pname;
    };
    mkApp = bin: {
      type = "app";
      program = "${self.packages.${system}.amc}/bin/${bin}";
    };
  in {
    apps.${system} = {
      amc-counter = mkApp "amc-counter";
      amc-todo = mkApp "amc-todo";
      amc-moves = mkApp "amc-moves";
      amc-automerge = mkApp "amc-automerge";
      bench = {
        type = "app";
        program = "${self.packages.${system}.bench}";
      };
    };

    packages.${system} = {
      default = self.packages.${system}.amc;

      amc = craneLib.buildPackage {
        inherit cargoArtifacts src pname;
      };

      amc-docs = craneLib.cargoDoc {
        inherit cargoArtifacts src pname;
      };

      bench = pkgs.writeShellScript "bench" ''
        PATH=${self.packages.${system}.amc}/bin:$PATH ${pkgs.python3.withPackages (ps: [ps.loguru])}/bin/python ${./bench.py}
      '';
    };

    formatter.${system} = pkgs.alejandra;

    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        (rust.override {
          extensions = ["rust-src"];
        })
        cargo-watch
        cargo-flamegraph
        cargo-release
        cargo-semver-checks

        python3
        python3Packages.pandas
        python3Packages.matplotlib
        python3Packages.seaborn

        black
      ];
    };
  };
}
