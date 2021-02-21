{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-20.09";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        hs = pkgs.haskellPackages;


        pkg = pkgs.haskell.lib.overrideCabal (hs.callCabal2nix "nix-expr-generator" ./. { }) {
          executableSystemDepends = [pkgs.fzf];
        };
      in {
        defaultPackage = pkg;
        packages = { inherit pkgs ; };
        devShell = pkg.env.overrideAttrs (super: {
          nativeBuildInputs = with pkgs; super.nativeBuildInputs ++ [
            hs.cabal-install
            ghcid
            hs.haskell-language-server
          ];
        });
      }
    );
}
