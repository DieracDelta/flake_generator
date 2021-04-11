#![allow(dead_code)]
#[warn(unused_variables)]
extern crate skim;
extern crate rowan;
use skim::prelude::*;
use std::io::Cursor;
use std::{env, error::Error, fs};

use smol_str::SmolStr;
use rnix::{types::*, NodeOrToken, SyntaxKind::*, SyntaxNode, NixLanguage};

enum UserAction {
    AddDep,
    RemoveDep,
    AddInput,
    RemoveInput,
    GenLib,
    GenBin(Lang),
}

enum Lang {
    Rust,
    Haskell,
    Python,
    JavaScript,
}

fn get_prompts(action: UserAction) -> Vec<Vec<String>> {
    match action {
        UserAction::AddDep =>
            vec![vec!["Add a dependency to your flake.".to_string(),
            "Please select an package from nixpkgs.".to_string()]],
            UserAction::RemoveDep =>
                vec![vec!["Remove a dependency from your flake. ".to_string(),
                "Please select a input to remove.".to_string()]],
                UserAction::AddInput =>
                    vec![vec!["Add an input to your flake.".to_string(),
                    "Please input a flake url and indicate if it's a flake".to_string()],
                    vec!["Is the input a flake?".to_string()]],
                    UserAction::RemoveInput =>
                        vec![vec!["Remove an input from your flake.".to_string(), "Please select an input to remove.".to_string()]],
                        UserAction::GenLib => unimplemented!(),
                        UserAction::GenBin(_) => unimplemented!()
    }
}

fn query_user_input(prompt: Vec<String>, items: Vec<String>) -> String {
    let agg = | x : Vec<String>| -> String {
        x.into_iter().rev()
            .fold("".to_string(),
            |acc, ele| -> String { format!("{}\n{}", acc, ele) })
    };
    let agg_prompt = agg(prompt);
    println!("prompt is:");
    println!("{}", &agg_prompt);
    let options = SkimOptionsBuilder::default()
        .header(Some(&agg_prompt))
        // a lot...
        .header_lines(50)
        .prompt(Some("Provide input:"))
        .inline_info(false)
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(agg(items)));
    Skim::run_with(&options, Some(items))
        .unwrap().query
}

fn get_str_val(node: &rowan::api::SyntaxNode<NixLanguage>) -> Result<String, String>{
    // TODO the clone is probably wrong
    Str::cast((*node).clone()).unwrap().parts().fold();
    println!("attr value is {:?}", Str::cast(cur_node_value).unwrap().parts());
}

/// TODO docstring
fn search_for_attr(attr: String, max_depth: usize,
    root_node: &rowan::api::SyntaxNode<NixLanguage>, exact_depth: Option<usize>)
    -> Result<Vec<rowan::api::SyntaxNode<NixLanguage>>, String> {
        // try to get out a attrset.
        // TODO the clone is probably wrong
        let mut stack = match AttrSet::cast((*root_node).clone()) {
            Some(rn) => rn.entries().collect(),
            None => Vec::new()
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

            let mut real_depth = cur_depth -1;
            let mut cur_node_attribute = "".to_string();
            // TODO url.url.url will break this
            for p in cur_node_key.path(){
                let tmp = Ident::cast(p).unwrap();
                let cur_attr = tmp.as_str();
                cur_node_attribute.push_str(".");
                cur_node_attribute.push_str(&cur_attr);
                real_depth += 1;
                if attr == cur_attr {
                    break;
                }
            }

            let is_match = cur_node_attribute == attr && exact_depth.map_or(true, |x| -> bool {x == real_depth});
            if is_match {
                result.push(cur_node_value);
            } else {
                match cur_node_value.kind() {
                    NODE_ATTR_SET => {
                        let cur_node_casted = AttrSet::cast(cur_node_value.clone()).unwrap();
                        if is_match {
                            result.push(cur_node_value);
                        } else {
                            cur_node_casted.entries()
                                .for_each(|entry| {
                                    stack.insert(0, entry);
                                    remaining_items_cur += 1;
                                });
                        }
                    }
                    NODE_STRING => {
                        // do nothing...
                        //println!("attr value is {:?}", Str::cast(cur_node_value).unwrap().parts());
                    }
                    _kind => () // println!("other kind: {:?}", _kind)
                }
            }
        }
        Ok(result)

    }


fn main() -> Result<(), Box<dyn Error>> {
    //query_user_input(get_prompts(UserAction::AddInput)[0].clone(), vec![]);

    let file = match env::args().skip(1).next() {
        Some(file) => file,
        None => {
            eprintln!("Usage: list-fns <file>");
            return Ok(());
        }
    };
    let content = fs::read_to_string(&file)?;
    let ast = rnix::parse(&content).as_result()?;
    // how to get it back
    //println!("{}", ast.node());
    let set = ast.root().inner().unwrap();
    let input_attrs = search_for_attr("inputs".to_string(), 1, &set, Some(0)).unwrap();
    let url_attrs = input_attrs.iter().fold(Vec::new(),
    | mut acc, ele | -> Vec<_> {
        acc.append(&mut search_for_attr("url".to_string(), 10, ele, None).unwrap()); acc
    });
    println!("urls found: {:?}", url_attrs);


    //for entry_wrapped in set.entries() {
    //let entry = entry_wrapped.value().unwrap();
    //println!("entry key is: {:?}",  entry_wrapped.key().unwrap().path().next().and_then(Ident::cast).unwrap().as_str());

    //match entry.kind() {
    //NODE_ATTR_SET => {
    ////println!("top attr is {:?}",  AttrSet::cast(entry).unwrap().key());
    //for input_kvs in AttrSet::cast(entry).unwrap().entries(){
    //// TODO do fold and separate into method
    //input_kvs.key().unwrap().path().for_each(|ele| {println!("key value is {:?}", Ident::cast(ele).unwrap().as_str())});
    //// morally speaking, should do this recursively, probably
    //println!("attr value is {:?}", input_kvs.value().and_then(Str::cast).unwrap().parts());
    //};
    //},
    //_ => {
    //println!("kind is {:?}", entry.kind());
    //println!("entry is {:?}", entry.text());
    //}
    //};
    //}

    Ok(())
}
