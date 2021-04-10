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

fn main() -> Result<(), Box<dyn Error>> {
    query_user_input(get_prompts(UserAction::AddInput)[0].clone(), vec![]);

    let file = match env::args().skip(1).next() {
        Some(file) => file,
        None => {
            eprintln!("Usage: list-fns <file>");
            return Ok(());
        }
    };
    let content = fs::read_to_string(&file)?;
    let ast = rnix::parse(&content).as_result()?;
    let set = ast.root().inner().and_then(AttrSet::cast).ok_or("root isn't a set")?;

    for entry in set.entries() {
        if let Some(lambda) = entry.value().and_then(Lambda::cast) {
            if let Some(attr) = entry.key() {
                let ident = attr.path().last().and_then(Ident::cast);
                let s = ident.as_ref().map_or("error", Ident::as_str);
                println!("Function name: {}", s);
                if let Some(comment) = find_comment(attr.node().clone()) {
                    println!("-> Doc: {}", comment);
                }

                let mut value = Some(lambda);
                while let Some(lambda) = value {
                    let ident = lambda.arg().and_then(Ident::cast);
                    let s = ident.as_ref().map_or("error", Ident::as_str);
                    println!("-> Arg: {}", s);
                    if let Some(comment) = lambda.arg().and_then(find_comment) {
                        println!("--> Doc: {}", comment);
                    }
                    value = lambda.body().and_then(Lambda::cast);
                }
                println!();
            }
        }
    }

    Ok(())
}

fn find_comment(node: SyntaxNode) -> Option<String> {
    let mut node = NodeOrToken::Node(node);
    let mut comments = Vec::new();
    loop {
        loop {
            if let Some(new) = node.prev_sibling_or_token() {
                node = new;
                break;
            } else {
                node = NodeOrToken::Node(node.parent()?);
            }
        }

        match node.kind() {
            TOKEN_COMMENT => match &node {
                NodeOrToken::Token(token) => comments.push(SmolStr::new(token.text())),
                NodeOrToken::Node(_) => unreachable!(),
            },
            t if t.is_trivia() => (),
            _ => break,
        }
    }
    let doc = comments
        .iter()
        .map(|it| it.trim_start_matches('#').trim())
        .collect::<Vec<_>>()
        .join("\n        ");
    return Some(doc).filter(|it| !it.is_empty());
}

