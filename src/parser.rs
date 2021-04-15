use rnix::{types::*, NixLanguage, StrPart, SyntaxKind::*};
use rowan::{GreenNodeBuilder, SyntaxNode};

fn kill_node(
    node: &rowan::api::SyntaxNode<NixLanguage>,
) -> Result<rowan::api::SyntaxNode<NixLanguage>, String> {
    // TODO replace 2 with TOKEN_WHITESPACE
    //let newnode = GreenNode::new(rowan::SyntaxKind(2), vec![].iter());
    let mut new_node = GreenNodeBuilder::new();
    new_node.start_node(rowan::SyntaxKind(2));
    new_node.finish_node();
    let mut new_root = SyntaxNode::<NixLanguage>::new_root(node.replace_with(new_node.finish()));
    loop {
        println!("did one iteration of the inner loop");
        if let Some(parent) = new_root.parent() {
            new_root = parent;
        } else {
            break;
        }
    }
    Ok(new_root)
}

fn get_str_val(node: &rowan::api::SyntaxNode<NixLanguage>) -> Result<String, String> {
    Ok(Str::cast((*node).clone()).unwrap().parts().iter().fold(
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

fn search_for_attr(
    attr: String,
    max_depth: usize,
    root_node: &rowan::api::SyntaxNode<NixLanguage>,
    exact_depth: Option<usize>,
) -> Result<Vec<(rowan::api::SyntaxNode<NixLanguage>, String, usize)>, String> {
    // assuming that the root node is an attrset
    let mut stack = match AttrSet::cast((*root_node).clone()) {
        Some(rn) => rn.entries().map(|x| (x, String::new())).collect(),
        None => Vec::new(),
    };

    let mut remaining_items_prev: usize = stack.len();
    let mut remaining_items_cur: usize = 0;
    let mut cur_depth = 0;
    let mut result = Vec::new();
    while !stack.is_empty() {
        let (cur_node, mut path) = stack.pop().unwrap();
        let cur_node_value = cur_node.value().unwrap();
        let cur_node_key = cur_node.key().unwrap();
        if remaining_items_prev == 0 {
            remaining_items_prev = remaining_items_cur - 1;
            remaining_items_cur = 0;
            cur_depth += 1;
        }
        if cur_depth > max_depth {
            //return Err(format!("Attribute {} does not exist at depth {:?}", attr, max_depth));
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
            && exact_depth.map_or(true, |x| -> bool { x == real_depth });
        if is_match {
            result.push((cur_node_value, path, real_depth));
        } else {
            match cur_node_value.kind() {
                NODE_ATTR_SET => {
                    let cur_node_casted = AttrSet::cast(cur_node_value.clone()).unwrap();
                    if is_match {
                        result.push((cur_node_value, path, real_depth));
                    } else {
                        cur_node_casted.entries().for_each(|entry| {
                            stack.insert(0, (entry, path.clone()));
                            remaining_items_cur += 1;
                        });
                    }
                }
                NODE_STRING => {
                    // TODO this should morally speaking *actually* fail
                    println!("nothing fail");
                }
                _kind => (), // println!("other kind: {:?}", _kind)
            }
        }
    }
    Ok(result)
}

pub fn get_inputs(root: &rowan::api::SyntaxNode<NixLanguage>) -> Vec<String> {
    let input_attrs = search_for_attr("inputs".to_string(), 1, root, None).unwrap();
    input_attrs.iter().fold(
        Vec::new(),
        |mut acc, (ele, _attribute_path, depth)| -> Vec<String> {
            let expected_depth = 3;
            match ele.kind() {
                // edge case of entire attribute set at once. E.g. inputs.nixpkgs.url =
                NODE_STRING => {
                    if *depth == expected_depth {
                        let result = Str::cast(ele.clone()).unwrap().parts().iter().fold(
                            String::new(),
                            |mut i_acc, i_ele| {
                                if let StrPart::Literal(s) = i_ele {
                                    i_acc.push_str(s)
                                }
                                i_acc
                            },
                        );
                        acc.push(result);
                    }
                    acc
                }
                NODE_ATTR_SET => search_for_attr("url".to_string(), 2, ele, None)
                    .unwrap()
                    .iter()
                    .fold(
                        acc,
                        |mut n_acc: Vec<String>, (n_ele, _n_path, n_depth)| -> Vec<String> {
                            if depth + n_depth == expected_depth {
                                let mut result = "".to_string();
                                result.push_str(&get_str_val(&n_ele).unwrap());
                                n_acc.push(result);
                            }
                            n_acc
                        },
                    ),
                _ => acc,
            }
        },
    )
}
