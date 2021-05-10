use crate::parser::input_utils::{merge_attr_sets, new_attr_set, new_key, new_string, wrap_root};
use crate::parser::utils::{node_to_string, string_to_node};
use crate::SmlStr;
use crate::SyntaxStructure;

use rnix::{types::*, SyntaxKind::*};

#[test]
pub fn check_new_string() {
    let phrase = "hello_world".to_string();
    let n = new_string(phrase.clone());
    let root = wrap_root(n);
    let result = Root::cast(root).unwrap();
    //let ast = rnix::parse(&phrase)
    //.as_result()
    //.map(|ast| ast.root()).unwrap();
    //assert_eq!(format!("{}", result.dump()).trim(), format!("{}", ast.dump()).trim())
    assert_eq!(format!("{}", result.dump()).trim(), "NODE_ROOT 0..13 {\n  NODE_STRING 0..13 {\n    TOKEN_STRING_START(\"\\\"\") 0..1\n    TOKEN_STRING_CONTENT(\"hello_world\") 1..12\n    TOKEN_STRING_END(\"\\\"\") 12..13\n  }\n}")
}

#[test]
pub fn check_new_attr_set() {
    let attrset = vec![
        (
            SyntaxStructure::Key(SmlStr::new_inline("test1")).into(),
            SyntaxStructure::StringLiteral(SmlStr::new_inline("value1")).into(),
        ),
        (
            SyntaxStructure::Key(SmlStr::new_inline("test2")).into(),
            SyntaxStructure::StringLiteral(SmlStr::new_inline("value2")).into(),
        ),
    ];
    let result = new_attr_set(attrset);
    let root = wrap_root(result);
    // TODO separate this out into a dump ast method..
    //let result = Root::cast(root).unwrap();
    //let r_string = format!("{}", result.dump());
    //let r_string = r_string;
    //println!("ast: {}", r_string.clone());
    assert_eq!(
        root.to_string(),
        "{\n\"test1\" = \"value1\";\n\"test2\" = \"value2\";\n}"
    );
}

#[test]
pub fn check_merge_attr_set() {
    let attrset = vec![
        (
            SyntaxStructure::Key(SmlStr::new_inline("test1")).into(),
            SyntaxStructure::StringLiteral(SmlStr::new_inline("value1")).into(),
        ),
        (
            SyntaxStructure::Key(SmlStr::new_inline("test2")).into(),
            SyntaxStructure::StringLiteral(SmlStr::new_inline("value2")).into(),
        ),
    ];
    let result = new_attr_set(attrset);
    let merged = merge_attr_sets(result.clone(), result);
    let root = wrap_root(merged);
    assert_eq!(
        root.to_string(),
        "{\n\"test1\" = \"value1\";\n\n\"test2\" = \"value2\";\n\n\n\"test1\" = \"value1\";\n\"test2\" = \"value2\";\n}"
    );
}
