use crate::parser::utils::{string_to_node, NixNode};
use anyhow::{anyhow, bail};
use either::Either;
use rnix::{types::*, NixLanguage, StrPart, SyntaxKind::*, parse};
use rowan::{api::SyntaxNode, GreenNode, GreenNodeBuilder, Language, NodeOrToken, GreenToken};
use std::string::ToString;
use nixpkgs_fmt::reformat_node;
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Input {
    url: String,
    is_flake: bool,
}

pub fn wrap_root(node: GreenNode) -> NixNode {
    let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_ROOT);
    let root = GreenNode::new(kind, (vec![NodeOrToken::Node(node)]));
    SyntaxNode::new_root(root)
}

pub fn new_string(s : String) -> GreenNode {
    let mut node = GreenNodeBuilder::new();
    let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_STRING);
    node.start_node(kind);
    let start_string_kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(TOKEN_STRING_START);
    node.token(start_string_kind, "\"");
    let string_content: rowan::SyntaxKind = NixLanguage::kind_to_raw(TOKEN_STRING_CONTENT);
    node.token(string_content, &s);
    let end_string_kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(TOKEN_STRING_END);
    node.token(end_string_kind, "\"");
    node.finish_node();
    node.finish()
}

pub fn new_key(s : String) -> GreenNode {
    let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_KEY);
    let children = vec![NodeOrToken::Node(new_string(s))];
    GreenNode::new(kind, children)
}

pub fn gen_key_value(key: String, value: String) -> GreenNode {
    let key_node : GreenNode = new_key(key);
    let value_node : GreenNode = new_string(value);
    new_key_value(key_node, value_node)
}

pub fn new_key_value(key : GreenNode, value: GreenNode) -> GreenNode {
    let kind = NixLanguage::kind_to_raw(NODE_KEY_VALUE);
    let assign_kind = NixLanguage::kind_to_raw(TOKEN_ASSIGN);
    let whitespace_kind = NixLanguage::kind_to_raw(TOKEN_WHITESPACE);
    let semicolon_kind = NixLanguage::kind_to_raw(TOKEN_SEMICOLON);
    let children = vec![
        NodeOrToken::Node(key),
        NodeOrToken::Token(GreenToken::new(whitespace_kind, " ")),
        NodeOrToken::Token(GreenToken::new(assign_kind, "=")),
        NodeOrToken::Token(GreenToken::new(whitespace_kind, " ")),
        NodeOrToken::Node(value),
        NodeOrToken::Token(GreenToken::new(semicolon_kind, ";")),
    ];
    GreenNode::new(kind, children)
}

pub fn gen_attr_set(attr_pairs: Vec<(String, String)>) -> GreenNode{
    let new_attr_pairs : Vec<(GreenNode, GreenNode)> =
        attr_pairs.iter().map(|(key, value)| (new_key(key.to_string()), new_string(value.to_string()))).collect();
    new_attr_set(new_attr_pairs)
}

// TODO give all the tokens their own constructors
/// inputs are: [(lhs, rhs), ...]
pub fn new_attr_set(attr_pairs: Vec<(GreenNode, GreenNode)>) -> GreenNode {
    let pairs : Vec<NodeOrToken<_,_>> = attr_pairs
        .iter()
        .map(move |(k, v)| {
            NodeOrToken::Node(new_key_value(k.clone(), v.clone()))
        }).collect::<Vec<_>>();
    let open_curly_kind = NixLanguage::kind_to_raw(TOKEN_CURLY_B_OPEN);
    let close_curly_kind = NixLanguage::kind_to_raw(TOKEN_CURLY_B_CLOSE);
    let whitespace_kind = NixLanguage::kind_to_raw(TOKEN_WHITESPACE);
    let attr_set_kind = NixLanguage::kind_to_raw(NODE_ATTR_SET);
    let mut token_vec = Vec::new();
    token_vec.push(vec![NodeOrToken::Token(GreenToken::new(open_curly_kind, "{"))]);
    token_vec.push(pairs);
    token_vec.push(vec![NodeOrToken::Token(GreenToken::new(close_curly_kind, "}"))]);
    let tokens = token_vec.iter().flatten().cloned().collect::<Vec<NodeOrToken<_,_>>>();
    GreenNode::new(attr_set_kind, tokens)
}

impl From<Input> for GreenNode {
    fn from(item: Input) -> Self {
        let inputs = vec![
            ("url".to_string(), item.url),
            ("flake".to_string(), item.is_flake.to_string())
        ];
        gen_attr_set(inputs)
    }
}
