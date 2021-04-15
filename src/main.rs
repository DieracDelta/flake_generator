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
    let mut user_data = UserMetadata::default();
    let mut cur_action : UserAction = UserAction::Intro;
    while cur_action != UserAction::Exit {
        let prompts : Vec<String> = get_prompts(cur_action);
        let prompt_items : Vec<String> = get_prompt_items(cur_action, &user_data);
        let user_selection : String = query_user_input(prompts, prompt_items, user_data.modify_existing);
        match cur_action {
            UserAction::Intro => {
                if user_selection == "create".to_string() {
                    // TODO prompt for a name
                    unimplemented!();
                } else if user_selection == "modify".to_string() {
                    user_data.modify_existing = true;
                    cur_action = UserAction::ModifyExisting
                } else {
                    unimplemented!();
                }
            }
            UserAction::ModifyExisting => {
                let content = fs::read_to_string(user_selection)?;
                let ast = rnix::parse(&content).as_result()?;
                user_data.root = Some(ast.root().inner().unwrap());
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
                // TODO better error handling
                //root.unwrap():
                cur_action = UserAction::Exit;
            }
            _ => unimplemented!(),
        }
    }

    Ok(())
}
