{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = (pkgs.rust-bin.nightly."2023-01-10".default.override {
          extensions = [
            "rust-src"
            "cargo"
            "rustc"
            "clippy"
            "rustfmt"
            "rust-analyzer"
            "rustc-dev"
          ];
        });
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.cargo-expand
          ];
          PATH_TO_RUST = "${rust}";
        };
      }
    );
}
