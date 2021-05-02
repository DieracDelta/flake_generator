use crate::parser::input_utils::{new_string, wrap_root, new_key, gen_attr_set};
use crate::parser::utils::{node_to_string, string_to_node};

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
    let attrset = vec![("test1".to_string(), "value1".to_string()), ("test2".to_string(), "value2".to_string())];
    let result = gen_attr_set(attrset);
    let root = wrap_root(result);
    // TODO separate this out into a dump ast method..
    //let result = Root::cast(root).unwrap();
    //let r_string = format!("{}", result.dump());
    //let r_string = r_string;
    //println!("ast: {}", r_string.clone());
    assert_eq!(root.to_string(), "{\"test1\" = \"value1\";\"test2\" = \"value2\";}");
}

