{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    {
      self,
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
        rootPath = ./.;
      in
      {
        formatter = treefmtEval.config.build.wrapper;
        packages = rec {
          madaha = naersk-lib.buildPackage {
            src = rootPath;
          };
          default = madaha;
        };
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              tokei
              cargo
              rustc
              rustfmt
              rust-analyzer

              pkg-config
              gcc
              jq

              # libs
              alsa-lib
              pipewire
              libclang
              jack2
            ];
            RUST_BACKTRACE = 1;
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            LIBCLANG_PATH = "${libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${glibc.dev}/include -isystem ${libclang.lib}/lib/clang/${libclang.version}/include";
          };
        devShells.default = self.devShell."${system}";
      }
    );
}
