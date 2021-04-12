#![allow(dead_code)]
extern crate rowan;
#[warn(unused_variables)]
use std::{error::Error, fs};
mod user;
use user::*;
use rnix::{types::*, NixLanguage};
mod parser;
use parser::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut root_root;
    let mut root: Option<&rowan::api::SyntaxNode<NixLanguage>> = None;
    let mut cur_action = UserAction::Intro;
    let mut modify_files = false;
    while cur_action != UserAction::Exit {
        let prompts = get_prompts(cur_action);
        let prompt_items = get_prompt_items(cur_action, root);
        let user_selection = query_user_input(prompts, prompt_items, modify_files);
        match cur_action {
            UserAction::Intro => {
                if user_selection == "create".to_string() {
                    // TODO prompt for a name
                } else if user_selection == "modify".to_string() {
                    modify_files = true;
                    cur_action = UserAction::ModifyExisting
                } else {
                    //println!("User selection was {}", user_selection);
                    unimplemented!();
                }
            }
            UserAction::ModifyExisting => {
                // try to parse
                //println!("selected file is: {}", user_selection);
                let content = fs::read_to_string(user_selection)?;
                let ast = rnix::parse(&content).as_result()?;
                // if it fails to parse, get the error
                root_root = ast.root().inner().unwrap();
                root = Some(&root_root);
                modify_files = false;
                cur_action = UserAction::IntroParsed;
            }
            UserAction::IntroParsed => {
                if user_selection == "Delete existing input" {
                    cur_action = UserAction::RemoveInput;
                } else if user_selection == "Add input" {
                    cur_action = UserAction::AddInput;
                }
            }
            UserAction::RemoveInput => {
                cur_action = UserAction::Exit;
            }
            _ => unimplemented!(),
        }
    }

    Ok(())
}
