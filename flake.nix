{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-20.09";
    utils.url = "github:numtide/flake-utils";
  };

  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
  };

  inputs.naersk.url = "github:nmattia/naersk";

  outputs = { self, nixpkgs, utils, rust-overlay, naersk }:
  utils.lib.eachDefaultSystem (system:
  let pkgs = import nixpkgs {
    inherit system;
    overlays = [
      rust-overlay.overlay
      (self: super: {
        rustc = self.latest.rustChannels.nightly.rust;
        cargo = self.latest.rustChannels.nightly.rust;
      })
    ];
  };
  naersk-lib = naersk.lib."${system}".override {
    rustc = pkgs.rustc;
    cargo = pkgs.cargo;
    rustfmt = pkgs.rustfmt;
  };
  in rec {
    packages.flake-generator = naersk-lib.buildPackage {
      pname = "flake-generator";
      root = ./.;
      #buildInputs = with pkgs; [skim];
    };
    defaultPackage = packages.flake-generator;

    apps.flake-generator = utils.lib.mkApp {
      drv = packages.flake-generator;
    };
    defaultApp = apps.flake-generator;

    devShell = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [ rustc cargo rustfmt];
    };
  });
}
