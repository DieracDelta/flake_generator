#![allow(dead_code)]
#[warn(unused_variables)]
extern crate skim;
use skim::prelude::*;
use std::io::Cursor;
use std::{env, error::Error, fs};

use smol_str::SmolStr;
use rnix::{types::*, NodeOrToken, SyntaxKind::*, SyntaxNode};

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

fn search_for_attr(attr: String, depth: usize, root_node: &AttrSet) -> Result<(&AttrSet, String), &str> {
    let stack: Vec<u64> = root_node.entries().into_vec();
    let remaining_items: usize;
    //for entry_wrapped in set.entries() {
        //let entry = entry_wrapped.value().unwrap();

    //}

    Err("unimplemented")
}


fn main() -> Result<(), Box<dyn Error>> {
    //query_user_input(get_prompts(UserAction::AddInput)[0].clone(), vec![]);

    let file = match env::args().skip(1).next() {
        Some(file) => file,
        None => {
            eprintln!("Usage: list-fns <file>");
            Ok(())
        }
    };
    let content = fs::read_to_string(&file)?;
    let ast = rnix::parse(&content).as_result()?;
    // how to get it back
    //println!("{}", ast.node());
    let set = ast.root().inner().and_then(AttrSet::cast).ok_or("root isn't a set")?;
    println!("set is: {:?}", set);

    for entry_wrapped in set.entries() {
        let entry = entry_wrapped.value().unwrap();
        println!("entry key is: {:?}",  entry_wrapped.key().unwrap().path().next().and_then(Ident::cast).unwrap().as_str());

        match entry.kind() {
            NODE_ATTR_SET => {
                //println!("top attr is {:?}",  AttrSet::cast(entry).unwrap().key());
                for input_kvs in AttrSet::cast(entry).unwrap().entries(){
                    // TODO do fold and separate into method
                    input_kvs.key().unwrap().path().for_each(|ele| {println!("key value is {:?}", Ident::cast(ele).unwrap().as_str())});
                    // morally speaking, should do this recursively, probably
                    println!("attr value is {:?}", input_kvs.value().and_then(Str::cast).unwrap().parts());
                };
            },
            _ => {
                println!("kind is {:?}", entry.kind());
                println!("entry is {:?}", entry.text());
            }
        };
    }

    Ok(())
}
