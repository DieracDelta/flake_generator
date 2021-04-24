use crate::parser::parser_utils::{
    get_inputs, get_output_node, node_to_string, remove_input, remove_input_from_output_fn,
    string_to_node, NixNode,
};

use rnix::{types::*, NixLanguage, StrPart, SyntaxKind::*};

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
        let result = remove_input(&ast, ".inputs.hello.url", None).unwrap();
        let result = remove_input(&result, ".inputs.another_one.url", None).unwrap();

        println!("{}", result.to_string());
        let new_inputs = get_inputs(&result);
        let deleted_hello = new_inputs.get(".inputs.hello.url");
        assert_eq!(deleted_hello, None);
        let deleted_another_one = new_inputs.get(".inputs.another_one.url");
        assert_eq!(deleted_another_one, None);
        assert_eq!(new_inputs.keys().len(), 0);

        // output fn arg tests
        let args = get_output_node(&result).unwrap().arg().unwrap();
        assert!(!args.clone().to_string().contains(','));
        match args.kind() {
            NODE_PATTERN => {
                let pattern = Pattern::cast(args).unwrap();
                let entries = pattern.entries().collect::<Vec<_>>();
                assert_eq!(entries.len(), 0);
            }
            _ => (),
        }
    }

    #[test]
    pub fn remove_inputs_one_arg() {
        let ast = string_to_node(include_str!("../../test_data/one_arg.nix").to_string()).unwrap();
        let result = remove_input(&ast, ".inputs.hello.url", None).unwrap();
        let result = remove_input(&result, ".inputs.another_one.url", None).unwrap();
        println!("{}", result.to_string());
        let new_inputs = get_inputs(&result);
        let deleted_hello = new_inputs.get(".inputs.hello.url");
        assert_eq!(deleted_hello, None);
        let deleted_another_one = new_inputs.get(".inputs.another_one.url");
        assert_eq!(deleted_another_one, None);
        assert_eq!(new_inputs.len(), 0);
    }
}
