#![feature(iter_intersperse)]
use crate::parser::utils::{string_to_node, NixNode};
use crate::SmlStr;
use anyhow::{anyhow, bail};
use either::Either;
use nixpkgs_fmt::reformat_node;
use rnix::{parse, types::*, NixLanguage, StrPart, SyntaxKind::*};
use rowan::{
    api::SyntaxNode, GreenNode, GreenNodeBuilder, GreenNodeData, GreenToken, Language, NodeOrToken,
};
use std::borrow::Borrow;
use std::ops::Deref;
use std::string::ToString;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Input {
    pub(crate) name: Option<SyntaxStructure>,
    pub(crate) url: Option<SyntaxStructure>,
    pub(crate) is_flake: Option<SyntaxStructure>,
}

pub fn wrap_root(node: GreenNode) -> NixNode {
    let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_ROOT);
    let root = GreenNode::new(kind, vec![NodeOrToken::Node(node)]);
    SyntaxNode::new_root(root)
}

pub fn new_string(s: String) -> GreenNode {
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
//TOKEN_WHITESPACE(" ") 18..19
//NODE_IDENT 19..23 {
//TOKEN_IDENT("true") 19..23
//}
//TOKEN_SEMICOLON(";") 23..

pub fn new_bool_literal(b: bool) -> GreenNode {
    let mut node = GreenNodeBuilder::new();
    let token_ident_kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(TOKEN_IDENT);
    let node_ident_kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_IDENT);
    node.start_node(node_ident_kind);
    node.token(token_ident_kind, b.to_string().as_str());
    node.finish_node();
    node.finish()
}

pub fn new_key(s: String) -> GreenNode {
    let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_KEY);
    let children = vec![NodeOrToken::Node(new_string(s))];
    GreenNode::new(kind, children)
}

pub fn gen_key_value(key: String, value: String) -> GreenNode {
    let key_node: GreenNode = new_key(key);
    let value_node: GreenNode = new_string(value);
    new_key_value(key_node, value_node)
}

pub fn new_key_value(key: GreenNode, value: GreenNode) -> GreenNode {
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
        NodeOrToken::Token(GreenToken::new(whitespace_kind, "\n")),
    ];
    GreenNode::new(kind, children)
}

// TODO merge with new_
pub fn gen_attr_set(attr_pairs: Vec<(String, String)>) -> GreenNode {
    let new_attr_pairs: Vec<(GreenNode, GreenNode)> = attr_pairs
        .iter()
        .map(|(key, value)| (new_key(key.to_string()), new_string(value.to_string())))
        .collect();
    new_attr_set(new_attr_pairs)
}

pub fn merge_attr_sets(a1: GreenNode, a2: GreenNode) -> GreenNode {
    let whitespace_kind = NixLanguage::kind_to_raw(TOKEN_WHITESPACE);
    let token = GreenToken::new(whitespace_kind, "\n");
    let delimiter = NodeOrToken::Token(token);
    let mut nodes = a2
        .children()
        .into_iter()
        .filter(|node| node.kind() == NixLanguage::kind_to_raw(NODE_KEY_VALUE))
        .map(|x| match x {
            NodeOrToken::Node(x) => NodeOrToken::Node(x.clone()),
            NodeOrToken::Token(x) => NodeOrToken::Token(x.clone()),
        })
        .flat_map(|x| vec![delimiter.clone(), x])
        .collect::<Vec<_>>();
    nodes.push(delimiter);
    let idx = a1
        .children()
        .position(|x| x.kind() == NixLanguage::kind_to_raw(TOKEN_CURLY_B_OPEN))
        .unwrap()
        + 1;
    a1.splice_children(idx..idx, nodes)
}

// TODO give all the tokens their own constructors
/// inputs are: [(lhs, rhs), ...]
pub fn new_attr_set(attr_pairs: Vec<(GreenNode, GreenNode)>) -> GreenNode {
    let pairs: Vec<NodeOrToken<_, _>> = attr_pairs
        .iter()
        .map(move |(k, v)| NodeOrToken::Node(new_key_value(k.clone(), v.clone())))
        .collect::<Vec<_>>();
    let open_curly_kind = NixLanguage::kind_to_raw(TOKEN_CURLY_B_OPEN);
    let close_curly_kind = NixLanguage::kind_to_raw(TOKEN_CURLY_B_CLOSE);
    let attr_set_kind = NixLanguage::kind_to_raw(NODE_ATTR_SET);
    let whitespace_kind = NixLanguage::kind_to_raw(TOKEN_WHITESPACE);
    let mut token_vec = Vec::new();
    token_vec.push(vec![NodeOrToken::Token(GreenToken::new(
        open_curly_kind,
        "{",
    ))]);
    token_vec.push(vec![NodeOrToken::Token(GreenToken::new(
        whitespace_kind,
        "\n",
    ))]);
    token_vec.push(pairs);
    token_vec.push(vec![NodeOrToken::Token(GreenToken::new(
        close_curly_kind,
        "}",
    ))]);
    let tokens = token_vec
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<NodeOrToken<_, _>>>();
    GreenNode::new(attr_set_kind, tokens)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum SyntaxStructure {
    Key(SmlStr),
    StringLiteral(SmlStr),
    Bool(bool),
}

impl From<SyntaxStructure> for GreenNode {
    fn from(ss: SyntaxStructure) -> Self {
        match ss {
            SyntaxStructure::Key(k) => new_key(k.to_string()),
            SyntaxStructure::StringLiteral(sl) => new_string(sl.to_string()),
            SyntaxStructure::Bool(b) => new_bool_literal(b),
        }
    }
}

impl From<Input> for GreenNode {
    fn from(item: Input) -> Self {
        let mut inputs = Vec::new();
        if let Some(s) = item.url {
            inputs.push((
                SyntaxStructure::Key(SmlStr::new_inline("url")).into(),
                s.into(),
            ));
        }
        if let Some(s) = item.is_flake {
            inputs.push((
                SyntaxStructure::Key(SmlStr::new_inline("flake")).into(),
                s.into(),
            ))
        }
        let input_name = item.name.unwrap().into();
        let inner_nodes = new_attr_set(inputs);
        new_attr_set(vec![(input_name, inner_nodes)])
    }
}
