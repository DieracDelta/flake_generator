use rnix::{types::*, NixLanguage, StrPart, SyntaxKind::*};
use rowan::{api::SyntaxNode, GreenNodeBuilder};

pub(crate) type NixNode = SyntaxNode<NixLanguage>;

use std::collections::HashMap;

// this is decent as a general technique, but bad for specifics
pub fn kill_node(node: &NixNode) -> Result<NixNode, String> {
    let mut new_node = GreenNodeBuilder::new();
    new_node.start_node(rowan::SyntaxKind(node.kind() as u16));
    new_node.finish_node();
    let a = new_node.finish();
    let b = node.replace_with(a);
    let mut new_root = NixNode::new_root(b);
    while let Some(parent) = new_root.parent() {
        new_root = parent;
    }
    Ok(Root::cast(new_root).unwrap().inner().unwrap())
}

/// Precondition: node is a attribute and parent is an attribute set
/// (1) get parent attrset
/// (2) iterate through parent's children nodes, searching for node to delete
/// (4) return a modified tree with node deleted
/// if child node is not found in parent, something is very wrong
/// so error out/fail, and not in a graceful manner
pub fn kill_node_attribute(node: &NixNode) -> Result<NixNode, String> {
    let parent = node.parent().unwrap();
    match parent.kind() {
        NODE_ATTR_SET => {
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
            let new_parent = parent.green().remove_child(idx);
            let mut new_root = NixNode::new_root(parent.replace_with(new_parent));
            while let Some(parent) = new_root.parent() {
                new_root = parent;
            }
            let tmp = Root::cast(new_root).unwrap();
            Ok(tmp.inner().unwrap())
        }
        _ => Err("Precondition violated: parent was not attribute set.".to_string()),
    }
}

/// converts a AST node to a string.
fn get_str_val(node: &NixNode) -> Result<String, String> {
    Ok(Str::cast(node.clone()).unwrap().parts().iter().fold(
        String::new(),
        |mut acc, ele| -> String {
            match ele {
                StrPart::Literal(s) => {
                    acc.push_str(s);
                    acc
                }
                StrPart::Ast(_) => unimplemented!(),
            }
        },
    ))
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
) -> Result<Vec<(NixNode, String, usize)>, String> {
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
            return Ok(vec![]);
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
    Ok(result)
}

/// searches AST for input nodes
/// returns hashmap of value to node
/// for example { "github.com/foo/bar": Node(FooBar)}
pub fn get_inputs(root: &NixNode) -> HashMap<String, NixNode> {
    search_for_attr("inputs".to_string(), 1, root, None)
        .unwrap()
        .into_iter()
        .flat_map(|(ele, _attribute_path, depth)| {
            const EXPECTED_DEPTH: usize = 3;
            match ele.kind() {
                // edge case of entire attribute set at once. E.g. inputs.nixpkgs.url = "foo";
                NODE_STRING => {
                    if depth == EXPECTED_DEPTH {
                        vec![(get_str_val(&ele).unwrap(), ele)]
                    } else {
                        vec![]
                    }
                }
                // common case of { nixpkgs = { url = "foo"; }; }
                NODE_ATTR_SET => search_for_attr("url".to_string(), 2, &ele, None)
                    .unwrap()
                    .into_iter()
                    .filter_map(|(n_ele, _n_node, n_depth)| {
                        (depth + n_depth == EXPECTED_DEPTH)
                            .then(|| (get_str_val(&n_ele).unwrap(), n_ele))
                    })
                    .collect(),
                _ => vec![],
            }
        })
        .collect()
}

/// remove string from argument list
pub fn remove_fn_arg(argument_list: &NixNode, input: &String) -> Result<NixNode, String> {
    unimplemented!();
}

/// remove input node from outputs
/// if it's listed
pub fn remove_input_from_output(root: &NixNode, input: &NixNode) -> Result<NixNode, String> {
    if let Ok(output_node) = search_for_attr("outputs".to_string(), 1, root, None) {
        println!("output_node {:?}", output_node);
    }

    unimplemented!();
}
