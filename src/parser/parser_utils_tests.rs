use crate::parser::{
    file::string_to_node,
    parser_utils::{get_inputs, NixNode},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn inputs() {
        let ast = string_to_node(include_str!("../../test_data/inputs.nix").to_string()).unwrap();
        let inputs = get_inputs(&ast);
        let nixpkgs = inputs.get("github:NixOS/nixpkgs/nixpkgs-unstable").unwrap();
        let nixCargoIntegration = inputs.get("github:yusdacra/nix-cargo-integration").unwrap();
        let hello = inputs.get("abc").unwrap();
        let another_one = inputs.get("hello_world").unwrap();
        assert_eq!(nixpkgs.0, ".nixpkgs.url");
        assert_eq!(nixCargoIntegration.0, ".nixCargoIntegration.url");
        assert_eq!(hello.0, ".inputs.hello.url");
        assert_eq!(another_one.0, ".another_one.url");
    }

    #[test]
    pub fn multi_arg() {
        assert_eq!(13, 13);
    }

    #[test]
    pub fn no_args() {
        assert_eq!(13, 13);
    }

    #[test]
    pub fn one_arg() {
        assert_eq!(13, 13);
    }

    #[test]
    pub fn zero_arg() {
        assert_eq!(13, 13);
    }
}
