{
  description = "QR Tattoo Redirector";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, rust-overlay, crate2nix, ... }:
    let name = "armqr";
    in utils.lib.eachDefaultSystem (system:
      let
        # Imports
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlay
            (self: super: {
              rustc = self.rust-bin.nightly.latest.default;
              cargo = self.rust-bin.nightly.latest.default;
            })
          ];
        };
        inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
          generatedCargoNix;

        # Create the cargo2nix project
        project = pkgs.callPackage (generatedCargoNix {
          inherit name;
          src = ./.;
        }) {
          defaultCrateOverrides = pkgs.defaultCrateOverrides // {
            ${name} = oldAttrs:
              {
                inherit buildInputs nativeBuildInputs;
              } // buildEnvVars;
          };
        };

        # Configuration for the non-Rust dependencies
        buildInputs = with pkgs; [ openssl.dev makeWrapper ];
        nativeBuildInputs = with pkgs; [ rustc cargo pkgconfig nixpkgs-fmt ];
        buildEnvVars = {
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };
      in rec {
        packages = {
          default = packages.armqr;
          armqr = project.rootCrate.build;
        };

        nixosModules.default = import ./nixos-module.nix;

        apps = {
          armqr = utils.lib.mkApp {
            inherit name;
            drv = packages.armqr;
          };
          default = apps.armqr;
        };

        # `nix develop`
        devShell = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          RUST_SRC_PATH =
            "${pkgs.rust.packages.nightly.rustPlatform.rustLibSrc}";
        } // buildEnvVars;
      });
}
