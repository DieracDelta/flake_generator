use crate::parser::utils::{
    get_inputs, get_output_node, node_to_string, remove_input, string_to_node,
};

use rnix::{types::*, SyntaxKind::*};

#[test]
pub fn check_inputs() {
    let ast = string_to_node(include_str!("../../test_data/inputs.nix").to_string()).unwrap();
    let inputs = get_inputs(&ast);
    let nixpkgs = inputs.get(".inputs.nixpkgs.url").unwrap().clone();
    assert_eq!(
        node_to_string(nixpkgs),
        "github:NixOS/nixpkgs/nixpkgs-unstable"
    );

    let nix_cargo_integration = inputs
        .get(".inputs.nixCargoIntegration.url")
        .unwrap()
        .clone();
    assert_eq!(
        node_to_string(nix_cargo_integration),
        "github:yusdacra/nix-cargo-integration"
    );

    let hello = inputs.get(".inputs.hello.url").unwrap().clone();
    assert_eq!(node_to_string(hello), "abc");

    let another_one = inputs.get(".inputs.another_one.url").unwrap().clone();
    assert_eq!(node_to_string(another_one), "hello_world");
}

#[test]
pub fn remove_inputs_multi_arg() {
    let ast = string_to_node(include_str!("../../test_data/multi_arg.nix").to_string()).unwrap();
    let result = remove_input(&ast, ".inputs.hello.url", None).unwrap();
    let result = remove_input(&result, ".inputs.another_one.url", None).unwrap();

    println!("{:?}", result.to_string());

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
pub fn remove_inputs_one_arg_ellipsis() {
    let ast =
        string_to_node(include_str!("../../test_data/one_arg_ellipsis.nix").to_string()).unwrap();
    let result = remove_input(&ast, ".inputs.hello.url", None).unwrap();
    let result = remove_input(&result, ".inputs.another_one.url", None).unwrap();
    let new_inputs = get_inputs(&result);
    let deleted_hello = new_inputs.get(".inputs.hello.url");
    assert_eq!(deleted_hello, None);
    let deleted_another_one = new_inputs.get(".inputs.another_one.url");
    assert_eq!(deleted_another_one, None);
    assert_eq!(new_inputs.len(), 0);
    let args = get_output_node(&result).unwrap().arg().unwrap().to_string();
    assert!(!args.contains(','));
    assert!(args.contains('{'));
    assert!(args.contains('}'));
    assert!(args.contains("..."));
}

#[test]
pub fn remove_inputs_one_arg_no_ellipsis() {
    let ast = string_to_node(include_str!("../../test_data/one_arg_no_ellipsis.nix").to_string())
        .unwrap();
    let result = remove_input(&ast, ".inputs.hello.url", None).unwrap();
    let result = remove_input(&result, ".inputs.another_one.url", None).unwrap();
    let new_inputs = get_inputs(&result);
    let deleted_hello = new_inputs.get(".inputs.hello.url");
    assert_eq!(deleted_hello, None);
    let deleted_another_one = new_inputs.get(".inputs.another_one.url");
    assert_eq!(deleted_another_one, None);
    assert_eq!(new_inputs.len(), 0);
    let args = get_output_node(&result).unwrap().arg().unwrap().to_string();
    assert!(!args.contains(','));
    assert!(args.contains('{'));
    assert!(args.contains('}'));
    assert!(!args.contains("..."));
}
