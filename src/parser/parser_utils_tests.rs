use crate::parser::{
    file::string_to_node,
    parser_utils::{get_inputs, remove_input_from_output_fn, NixNode},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn get_inputs() {
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
