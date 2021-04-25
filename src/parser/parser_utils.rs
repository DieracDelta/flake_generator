use anyhow::{anyhow, bail};
use rnix::{types::*, NixLanguage, StrPart, SyntaxKind::*};
use rowan::api::{Language, SyntaxNode};

pub(crate) type NixNode = SyntaxNode<NixLanguage>;

use std::collections::HashMap;

/// Precondition: node is a attribute and parent is an attribute set
/// (1) get parent attrset
/// (2) iterate through parent's children nodes, searching for node to delete
/// (4) return a modified tree with node deleted
/// if child node is not found in parent, something is very wrong
/// so error out/fail, and not in a graceful manner
/// amount parameter specifies number of nodes/tokens to kill
pub fn kill_node_attribute(node: &NixNode, amount: usize) -> anyhow::Result<NixNode> {
    let parent = node.parent().unwrap();
    match parent.kind() {
        NODE_ATTR_SET | NODE_PATTERN => {
            let mut child_node_idxs =
                parent
                    .green()
                    .children()
                    .enumerate()
                    .filter_map(|(idx, val)| {
                        // the '.to_owned()' is required to turn GreenNodeData into GreenNode
                        // because GreenNodeData doesn't implement PartialEq
                        val.into_node().and_then(|inner_node| {
                            (*inner_node == node.green().to_owned()).then(|| idx)
                        })
                    });
            // only one child node should match
            let idx = child_node_idxs.next().expect("Child not in parent tree");
            assert!(
                child_node_idxs.next().is_none(),
                "AST in inconsistent state. Child found multiple times in parent tree."
            );
            let result = node
                .parent()
                .unwrap()
                .green()
                .splice_children(idx..idx + amount, std::iter::empty());
            let mut new_root = NixNode::new_root(parent.replace_with(result));
            while let Some(parent) = new_root.parent() {
                new_root = parent;
            }
            let tmp = Root::cast(new_root).unwrap();
            Ok(tmp.inner().unwrap())
        }
        _ => Err(anyhow!(
            "Precondition violated: parent was not attribute set.".to_string()
        )),
    }
}

/// converts a AST node to a string.
pub fn node_to_string(node: &NixNode) -> String {
    Str::cast(node.clone())
        .unwrap()
        .parts()
        .iter()
        .fold(String::new(), |mut acc, ele| -> String {
            match ele {
                StrPart::Literal(s) => {
                    acc.push_str(s);
                    acc
                }
                StrPart::Ast(_) => unimplemented!(),
            }
        })
}

pub fn string_to_node(content: String) -> anyhow::Result<NixNode> {
    let ast = match rnix::parse(&content).as_result() {
        Ok(parsed) => parsed,
        Err(err) => {
            bail!(format!("could not parse as a nix file: {}", err));
        }
    };
    Ok(ast.root().inner().unwrap())
}

/// given an attribute name, searches to max_depth
/// for the given attribute name
/// it is assumed that the node's max depth >= exact_depth
/// and root_node is of type attrset
/// returns a vector of tuples that match
/// (matching_node, path, depth)
/// Example: searching for `foo`
/// ```
/// "{\"foo\": \"bar\"}"
/// ```
/// Will return [bar_node, "foo", 1]
fn search_for_attr(
    attr: String,
    max_depth: usize,
    root_node: &NixNode,
    exact_depth: Option<usize>,
) -> Vec<(NixNode, String, usize)> {
    // assuming that the root node is an attrset
    // TODO if it's not, we should fail louder
    let mut stack = match AttrSet::cast((*root_node).clone()) {
        Some(rn) => rn.entries().map(|x| (x, String::new(), 0)).collect(),
        None => Vec::new(),
    };

    let mut result = Vec::new();

    while let Some((cur_node, mut path, cur_depth)) = stack.pop() {
        let cur_node_value = cur_node.value().unwrap();
        let cur_node_key = cur_node.key().unwrap();

        if cur_depth > max_depth {
            // failing softly here since we're past the max depth
            // might want to be a bit more loud
            return vec![];
        }

        let mut real_depth = cur_depth;
        let mut cur_node_attribute = "".to_string();
        let mut is_match = false;

        for p in cur_node_key.path() {
            let tmp = Ident::cast(p).unwrap();
            let cur_attr = tmp.as_str();
            cur_node_attribute.push('.');
            cur_node_attribute.push_str(&cur_attr);
            real_depth += 1;
            is_match = attr == cur_attr || is_match;
        }

        path.push_str(&cur_node_attribute);
        is_match = (is_match || cur_node_attribute == attr)
            && exact_depth.map_or(true, |x| x == real_depth);
        if is_match {
            result.push((cur_node_value, path, real_depth));
        } else {
            match cur_node_value.kind() {
                NODE_ATTR_SET => {
                    let cur_node_casted = AttrSet::cast(cur_node_value).unwrap();
                    stack.extend(
                        cur_node_casted
                            .entries()
                            .map(|entry| (entry, path.clone(), real_depth)),
                    );
                }
                _kind => (),
            }
        }
    }
    result
}

/// searches AST for input nodes
/// returns hashmap of value to node
/// for example { "github.com/foo/bar": Node(FooBar)}
pub fn get_inputs(root: &NixNode) -> HashMap<String, (String, NixNode)> {
    search_for_attr("inputs".to_string(), 1, root, None)
        .into_iter()
        .flat_map(|(ele, attribute_path, depth)| {
            // inputs.{}.url: we expect the depth to be 3
            const EXPECTED_DEPTH: usize = 3;
            match ele.kind() {
                // edge case of entire attribute set at once. E.g. inputs.nixpkgs.url = "foo";
                NODE_STRING => {
                    if depth == EXPECTED_DEPTH {
                        vec![(attribute_path, (node_to_string(&ele), ele))]
                    } else {
                        vec![]
                    }
                }
                // common case of { nixpkgs = { url = "foo"; }; }
                NODE_ATTR_SET => search_for_attr("url".to_string(), 2, &ele, None)
                    .into_iter()
                    .filter_map(|(n_ele, path, n_depth)| {
                        (depth + n_depth == EXPECTED_DEPTH).then(|| {
                            (
                                format!("{}{}", attribute_path, path),
                                (node_to_string(&n_ele), n_ele),
                            )
                        })
                    })
                    .collect(),
                _ => vec![],
            }
        })
        .collect()
}

// exists for test usage
pub fn get_output_node(root: &NixNode) -> anyhow::Result<Lambda> {
    Ok(Lambda::cast(
        search_for_attr("outputs".to_string(), 2, root, None)
            .get(0)
            .unwrap()
            .clone()
            .0,
    )
    .unwrap())
}

/// remove input node from outputs
/// if it's listed
pub fn remove_input_from_output_fn(root: &NixNode, input_name: &str) -> anyhow::Result<NixNode> {
    let output_node = search_for_attr("outputs".to_string(), 2, root, None);
    assert!(output_node.len() == 1);
    let output_fn_node = Lambda::cast(output_node.get(0).unwrap().0.clone()).unwrap();
    if let Some(args) = output_fn_node.arg() {
        match args.kind() {
            NODE_IDENT => Ok(root.clone()),
            NODE_PATTERN => {
                // TODO once rnix implements filter_entries, use that.
                let fn_args = Pattern::cast(args.clone()).unwrap();
                if fn_args.entries().next().is_none() {
                    return Ok((*root).clone());
                }
                let children = args.children_with_tokens();
                let mut matching_arg_nodes =
                    children.clone().enumerate().filter_map(|(idx, val)| {
                        val.as_node()
                            .and_then(|n| PatEntry::cast(n.clone()))
                            .and_then(|pat| {
                                (pat.name().unwrap().as_str() == input_name).then(|| (idx, pat))
                            })
                    });

                let (arg_node_idx, arg_node) = matching_arg_nodes.next().unwrap();
                assert!(
                    matching_arg_nodes.next().is_none(),
                    "Two of the same argument found. Error out!"
                );
                // if there's a comma after the argument, we need to
                // delete the comma and the argument
                // can have a comma if there's another argument after
                // or if it's the last argument and there are ellipsis
                let mut matching_comma = children.enumerate().filter_map(|(idx, val)| {
                    if idx > arg_node_idx && val.kind() == TOKEN_COMMA {
                        Some(idx)
                    } else {
                        None
                    }
                });
                // unwrap or zero
                let idx_end = match matching_comma.next() {
                    Some(idx) => idx - arg_node_idx + 1,
                    None => 1,
                };
                kill_node_attribute(&arg_node.node(), idx_end)
            }
            _ => unimplemented!(),
        }
    } else {
        Ok((*root).clone())
    }
}

pub fn remove_input(
    root: &NixNode,
    dead_node_name: &str,
    user_inputs: Option<HashMap<String, (String, NixNode)>>,
) -> anyhow::Result<NixNode> {
    let inputs = match user_inputs {
        Some(inputs) => inputs,
        None => get_inputs(root),
    };
    let (_, dead_node) = inputs.get(dead_node_name).unwrap();
    let new_root = match kill_node_attribute(&dead_node.parent().unwrap(), 1) {
        Ok(node) => node,
        Err(err) => bail!(format!("could not remove input: {}", err)),
    };
    let input_name = get_attr(1, dead_node_name).unwrap();

    remove_input_from_output_fn(&new_root, &input_name)
}

pub fn get_attr(depth: usize, full_path: &str) -> Option<&str> {
    full_path.split('.').rev().nth(depth)
}
