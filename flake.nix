{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      naersk,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        toolchain = pkgs.rust-bin.selectLatestNightlyWith (
          toolchain:
          toolchain.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "cargo"
              "rustc"
            ];
          }
        );
        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

        buildTwm =
          args:
          naersk'.buildPackage (
            {
              src = ./.;
              nativeBuildInputs = [ pkgs.installShellFiles ];

              postInstall = ''
                installShellCompletion --cmd twm \
                  --bash <($out/bin/twm --print-bash-completion) \
                  --zsh <($out/bin/twm --print-zsh-completion) \
                  --fish <($out/bin/twm --print-fish-completion)

                $out/bin/twm --print-man > twm.1
                installManPage twm.1

              '';
            }
            // args
          );

        twm = buildTwm { };

        twm-dev = buildTwm { release = false; };
      in
      {
        formatter = pkgs.nixpkgs-fmt;
        packages = {
          default = twm;
          twm = twm;
          twm-dev = twm-dev;
        };

        devShell =
          with pkgs;
          mkShell {
            buildInputs = with pkgs; [
              toolchain
              pkg-config
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            PKG_CONFIG_PATH = lib.makeBinPath [ pkg-config ];
          };
      }
    );
}
