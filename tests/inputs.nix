{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixCargoIntegration = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  inputs.hello.url = "abc";

  inputs.another_one = {
    url = "hello_world";
  };

  outputs = inputs@{...}: {};
}
