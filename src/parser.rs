use rnix::{types::*, NixLanguage, StrPart, SyntaxKind::*};
use crate::user::*;

fn get_str_val(node: &rowan::api::SyntaxNode<NixLanguage>) -> Result<String, String> {
    // TODO the clone is probably wrong
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

/// TODO docstring
fn search_for_attr(
    attr: String,
    max_depth: usize,
    root_node: &rowan::api::SyntaxNode<NixLanguage>,
    exact_depth: Option<usize>,
) -> Result<Vec<rowan::api::SyntaxNode<NixLanguage>>, String> {
    // *big* assumption that the root node is an attrset
    let mut stack = match AttrSet::cast((*root_node).clone()) {
        Some(rn) => rn.entries().collect(),
        None => Vec::new(),
    };
    // it's a tree so this works
    let mut remaining_items_prev: usize = stack.len();
    let mut remaining_items_cur: usize = 0;
    let mut cur_depth = 0;
    let mut result = Vec::new();
    while !stack.is_empty() {
        // TODO note that we're popping off the back
        let cur_node = stack.pop().unwrap();
        let cur_node_value = cur_node.value().unwrap();
        let cur_node_key = cur_node.key().unwrap();
        if remaining_items_prev == 0 {
            remaining_items_prev = remaining_items_cur - 1;
            remaining_items_cur = 0;
            cur_depth += 1;
        }
        if cur_depth > max_depth {
            //return Err(format!("Attribute {} does not exist at depth {:?}", attr, max_depth));
            return Ok(vec![]);
        }

        let mut real_depth = cur_depth - 1;
        let mut cur_node_attribute = "".to_string();
        // TODO url.url.url will break this
        let mut is_match = false;
        for p in cur_node_key.path() {
            let tmp = Ident::cast(p).unwrap();
            let cur_attr = tmp.as_str();
            cur_node_attribute.push_str(".");
            cur_node_attribute.push_str(&cur_attr);
            real_depth += 1;
            //println!("attr: {}", cur_attr);
            if attr == cur_attr {
                is_match = true;
            }
        }
        //println!("cur_node_attribute: {}", cur_node_attribute);

        is_match = (is_match || cur_node_attribute == attr)
            && exact_depth.map_or(true, |x| -> bool { x == real_depth });
        if is_match {
            result.push(cur_node_value);
        } else {
            match cur_node_value.kind() {
                NODE_ATTR_SET => {
                    let cur_node_casted = AttrSet::cast(cur_node_value.clone()).unwrap();
                    if is_match {
                        result.push(cur_node_value);
                    } else {
                        cur_node_casted.entries().for_each(|entry| {
                            stack.insert(0, entry);
                            remaining_items_cur += 1;
                        });
                    }
                }
                NODE_STRING => {
                    //println!("nothing fail");
                    // do nothing...
                    //println!("attr value is {:?}", Str::cast(cur_node_value).unwrap().parts());
                }
                _kind => (), // println!("other kind: {:?}", _kind)
            }
        }
    }
    Ok(result)
}


pub fn get_prompt_items(
    action: UserAction,
    root: Option<&rowan::api::SyntaxNode<NixLanguage>>,
) -> Vec<String> {
    match action {
        UserAction::Intro => vec!["create".to_string(), "modify".to_string()],
        UserAction::IntroParsed => {
            vec!["Delete existing input".to_string(), "Add input".to_string()]
        }
        UserAction::ModifyExisting => vec![],
        UserAction::RemoveInput => {
            let input_attrs =
                search_for_attr("inputs".to_string(), 1, &(root.unwrap()), None).unwrap();
            input_attrs
                .iter()
                .fold(Vec::new(), |acc, ele| -> Vec<String> {
                    match ele.kind() {
                        // TODO weird edge case
                        //NODE_STRING => {println!("ele: {}", ele.key().path()); acc},
                        NODE_ATTR_SET => search_for_attr("url".to_string(), 10, ele, None)
                            .unwrap()
                            .iter()
                            .fold(acc, |mut n_acc: Vec<String>, n_ele| -> Vec<String> {
                                n_acc.push(get_str_val(&n_ele).unwrap());
                                n_acc
                            }),
                        _ => unimplemented!(),
                    }
                })
        }
        _ => vec![],
    }
}

