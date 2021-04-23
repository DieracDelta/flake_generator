{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rustOverlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixCargoIntegration = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rustOverlay.follows = "rustOverlay";
    };
  };

  outputs = inputs@{nixpkgs, nixCargoIntegration, ...}: inputs.nixCargoIntegration.lib.makeOutputs {
    root = ./.;
    overrides = {
      common = prev: {
        env = prev.env // {
          TEMPLATER_FMT_BIN = "${prev.pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt";
          TEMPLATER_CARGO_BIN = "${prev.pkgs.rustc}/bin/cargo";
        };
      };
    };
  };
}
