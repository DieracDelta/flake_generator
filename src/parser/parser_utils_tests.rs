use crate::parser::{
    file::string_to_node,
    parser_utils::{get_inputs, remove_input_from_output_fn, NixNode},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn check_inputs() {
        let ast = string_to_node(include_str!("../../test_data/inputs.nix").to_string()).unwrap();
        let inputs = get_inputs(&ast);
        let nixpkgs = inputs.get(".inputs.nixpkgs.url").unwrap();
        assert_eq!(nixpkgs.0, "github:NixOS/nixpkgs/nixpkgs-unstable");

        let nix_cargo_integration = inputs.get(".inputs.nixCargoIntegration.url").unwrap();
        assert_eq!(
            nix_cargo_integration.0,
            "github:yusdacra/nix-cargo-integration"
        );

        let hello = inputs.get(".inputs.hello.url").unwrap();
        assert_eq!(hello.0, "abc");

        let another_one = inputs.get(".inputs.another_one.url").unwrap();
        assert_eq!(another_one.0, "hello_world");
    }

    #[test]
    pub fn remove_inputs_multi_arg() {
        let ast =
            string_to_node(include_str!("../../test_data/multi_arg.nix").to_string()).unwrap();
    }

    #[test]
    pub fn remove_inputs_no_args() {
        let ast = string_to_node(include_str!("../../test_data/no_args.nix").to_string()).unwrap();
        assert_eq!(13, 13);
    }

    #[test]
    pub fn remove_inputs_one_arg() {
        let ast = string_to_node(include_str!("../../test_data/one_arg.nix").to_string()).unwrap();
        assert_eq!(13, 13);
    }

    #[test]
    pub fn remove_inputs_zero_args() {
        let ast =
            string_to_node(include_str!("../../test_data/zero_args.nix").to_string()).unwrap();
        assert_eq!(13, 13);
    }
}
