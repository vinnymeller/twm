{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, naersk, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" "cargo" ];
        });
        naersk-lib = pkgs.callPackage naersk { };


        buildTwm = args: naersk-lib.buildPackage {
            src = ./.;
        } // args;

        twm = buildTwm {};

        twm-dev = buildTwm { release = false; };

        in
      {
        formatter = pkgs.nixpkgs-fmt;
        packages = {
            default = twm;
            twm = twm;
            twm-dev = twm-dev;
        };

        devShell = with pkgs; mkShell {
          buildInputs = [
            rustVersion
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };
      });
}
