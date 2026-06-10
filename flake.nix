{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    {
      nixpkgs,
      utils,
      naersk,
      treefmt-nix,
      ...
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
      in
      rec {
        formatter = treefmtEval.config.build.wrapper;
        packages = rec {
          madaha = naersk-lib.buildPackage {
            src = ./.;
          };
          default = madaha;
        };
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              rust-analyzer
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            CARGO_HOME = ".cargo";

          };
        devShells.default = devShell;
      }
    );
}
