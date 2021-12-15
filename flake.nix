{
  description = "A Commodore 64 emulator in Rust";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-21.05";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-version = "1.57.0";
        rust-stable = pkgs.rust-bin.stable.${rust-version}.default.override {
          extensions = [ "rust-src" ];
        };
      in
        {
          devShell = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rust-stable
            ];
          };
        }
    );
}
