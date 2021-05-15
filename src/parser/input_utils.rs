use crate::parser::utils::{string_to_node, NixNode};
use crate::SmlStr;
use rnix::{NixLanguage, SyntaxKind::*};
use rowan::{api::SyntaxNode, GreenNode, GreenNodeBuilder, GreenToken, Language, NodeOrToken};
use std::string::ToString;
use NodeOrToken::{Node, Token};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Input {
    pub(crate) name: Option<SyntaxStructure>,
    pub(crate) url: Option<SyntaxStructure>,
    pub(crate) is_flake: Option<SyntaxStructure>,
}

pub fn wrap_root(node: GreenNode) -> NixNode {
    let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_ROOT);
    let root = GreenNode::new(kind, vec![Node(node)]);
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

/// creates a bool literal. Example of what
/// this should look like structurally:
///
/// NODE_IDENT {
///   TOKEN_IDENT("true")
/// }
///TOKEN_SEMICOLON(";") 23..
pub fn new_bool_literal(b: bool) -> GreenNode {
    let mut node = GreenNodeBuilder::new();
    let token_ident_kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(TOKEN_IDENT);
    let node_ident_kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_IDENT);
    node.start_node(node_ident_kind);
    node.token(token_ident_kind, b.to_string().as_str());
    node.finish_node();
    node.finish()
}

pub fn new_key_value(key: GreenNode, value: GreenNode) -> GreenNode {
    let kind = NixLanguage::kind_to_raw(NODE_KEY_VALUE);
    let assign_kind = NixLanguage::kind_to_raw(TOKEN_ASSIGN);
    let whitespace_kind = NixLanguage::kind_to_raw(TOKEN_WHITESPACE);
    let semicolon_kind = NixLanguage::kind_to_raw(TOKEN_SEMICOLON);
    let children = vec![
        Node(key),
        Token(GreenToken::new(whitespace_kind, " ")),
        Token(GreenToken::new(assign_kind, "=")),
        Token(GreenToken::new(whitespace_kind, " ")),
        Node(value),
        Token(GreenToken::new(semicolon_kind, ";")),
        Token(GreenToken::new(whitespace_kind, "\n")),
    ];
    GreenNode::new(kind, children)
}

pub fn merge_attr_sets(a1: GreenNode, a2: GreenNode) -> GreenNode {
    let whitespace_kind = NixLanguage::kind_to_raw(TOKEN_WHITESPACE);
    let token = GreenToken::new(whitespace_kind, "\n");
    let delimiter = Token(token);
    let mut nodes = a2
        .children()
        .into_iter()
        .filter(|node| node.kind() == NixLanguage::kind_to_raw(NODE_KEY_VALUE))
        .map(|x| match x {
            Node(x) => Node(x.clone()),
            Token(x) => Token(x.clone()),
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
        .map(move |(k, v)| Node(new_key_value(k.clone(), v.clone())))
        .collect::<Vec<_>>();
    let open_curly_kind = NixLanguage::kind_to_raw(TOKEN_CURLY_B_OPEN);
    let close_curly_kind = NixLanguage::kind_to_raw(TOKEN_CURLY_B_CLOSE);
    let attr_set_kind = NixLanguage::kind_to_raw(NODE_ATTR_SET);
    let whitespace_kind = NixLanguage::kind_to_raw(TOKEN_WHITESPACE);
    let mut token_vec = Vec::new();
    token_vec.push(vec![Token(GreenToken::new(open_curly_kind, "{"))]);
    token_vec.push(vec![Token(GreenToken::new(whitespace_kind, "\n"))]);
    token_vec.push(pairs);
    token_vec.push(vec![Token(GreenToken::new(close_curly_kind, "}"))]);
    let tokens = token_vec
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<NodeOrToken<_, _>>>();
    GreenNode::new(attr_set_kind, tokens)
}

pub(crate) struct Key {
    val: SmlStr,
}

impl From<Key> for GreenNode {
    fn from(item: Key) -> Self {
        let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_KEY);
        let children = vec![Node(new_string(item.val.to_string()))];
        GreenNode::new(kind, children)
    }
}

pub(crate) struct StringLiteral {
    val: SmlStr,
}

impl From<StringLiteral> for GreenNode {
    fn from(item: StringLiteral) -> Self {
        let mut node = GreenNodeBuilder::new();
        let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_STRING);
        node.start_node(kind);
        let start_string_kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(TOKEN_STRING_START);
        node.token(start_string_kind, "\"");
        let string_content: rowan::SyntaxKind = NixLanguage::kind_to_raw(TOKEN_STRING_CONTENT);
        node.token(string_content, &item.val.to_string());
        let end_string_kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(TOKEN_STRING_END);
        node.token(end_string_kind, "\"");
        node.finish_node();
        node.finish()
    }
}

pub(crate) struct Bool {
    val: bool,
}

pub(crate) struct KeyValue {
    key: Key,
    val: StringLiteral,
}

/// rnix::ParsedType is not good for node/creation since it literally wraps
/// SyntaxNode. This is a solution for that
/// This isn't quite a bijection, but we should make a tryFrom implementation
/// both ways (bijection) that fails if SyntaxStructure does not have an analogue ParsedType
/// FIXME this should be nearly trivial given the from implementation with greennode
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum SyntaxStructure {
    KeyValue(Box<SyntaxStructure>, Box<SyntaxStructure>),
    Key(SmlStr),
    StringLiteral(SmlStr),
    Bool(bool),
}

impl From<SyntaxStructure> for GreenNode {
    fn from(ss: SyntaxStructure) -> Self {
        match ss {
            SyntaxStructure::Key(k) => (Key { val: k }).into(),
            SyntaxStructure::StringLiteral(sl) => new_string(sl.to_string()),
            SyntaxStructure::Bool(b) => new_bool_literal(b),
            SyntaxStructure::KeyValue(key, value) => new_key_value((*key).into(), (*value).into()),
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
