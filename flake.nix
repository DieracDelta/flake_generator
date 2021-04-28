{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixCargoIntegration = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ nixpkgs, nixCargoIntegration, ... }:
    nixCargoIntegration.lib.makeOutputs {
      root = ./.;
      buildPlatform = "crate2nix";
    };
}
