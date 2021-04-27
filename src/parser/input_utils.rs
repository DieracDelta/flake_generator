use crate::parser::utils::{string_to_node, NixNode};
use anyhow::{anyhow, bail};
use either::Either;
use rnix::{types::*, NixLanguage, StrPart, SyntaxKind::*};
use rowan::{api::SyntaxNode, GreenNode, GreenNodeBuilder, Language};
use std::convert::identity;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Input {
    url: String,
    is_flake: bool,
}

/// inputs are: [(lhs, rhs), ...]
fn create_attr(attr_pairs: Vec<(String, Either<String, NixNode>)>) -> GreenNode {
    attr_pairs
        .iter()
        .map(|(lhs, rhs)| (lhs, rhs.as_ref().either(identity, |x| x.to_string())));
    let mut node = GreenNodeBuilder::new();
    let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_ATTR_SET);
    node.start_node(NixLanguage::kind_to_raw(NODE_ATTR_SET));
    node.finish_node();
    node.finish()
}

impl From<Input> for GreenNode {
    fn from(item: Input) -> Self {
        let mut node = GreenNodeBuilder::new();
        let kind: rowan::SyntaxKind = NixLanguage::kind_to_raw(NODE_ATTR_SET);
        node.start_node(kind);
        node.finish_node();
        node.finish()
    }
}
