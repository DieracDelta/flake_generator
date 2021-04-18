mod parser;
mod user;

use std::{fs, str::FromStr};

use parser::kill_node_attribute;
use rnix::types::*;
use user::*;

struct ActionStack {
    inner: Vec<UserAction>,
}

impl ActionStack {
    fn new() -> Self {
        Self {
            inner: vec![UserAction::Intro],
        }
    }

    fn push(&mut self, action: UserAction) {
        self.inner.push(action)
    }

    fn pop(&mut self) -> UserAction {
        if self.inner.len() > 1 {
            self.inner.pop().unwrap()
        } else {
            *self.inner.last().unwrap()
        }
    }

    fn clear(&mut self) {
        self.inner.clear();
        self.inner.push(UserAction::Intro);
    }

    fn current(&self) -> UserAction {
        *self.inner.last().unwrap()
    }
}

fn main() {
    let mut user_data = UserMetadata::default();
    let mut action_stack = ActionStack::new();
    while action_stack.current() != UserAction::Exit {
        let cur_action = action_stack.current();
        let user_selection = UserPrompt::from_str(&user_data.get_user_result(cur_action)).unwrap();
        match user_selection {
            UserPrompt::Back => {
                action_stack.pop();
            }
            UserPrompt::Exit => break,
            UserPrompt::Create => todo!("implement create; prompt for a name"),
            UserPrompt::Modify => {
                user_data.modify_existing = true;
                action_stack.push(UserAction::ModifyExisting);
            }
            UserPrompt::DeleteInput => action_stack.push(UserAction::RemoveInput),
            UserPrompt::AddInput => action_stack.push(UserAction::AddInput),
            UserPrompt::Other(other) => {
                match cur_action {
                    UserAction::ModifyExisting => {
                        let content = match fs::read_to_string(&other) {
                            Ok(content) => content,
                            Err(err) => {
                                const IS_DIRECTORY_ERRNO: i32 = 21;
                                if let Some(IS_DIRECTORY_ERRNO) = err.raw_os_error() {
                                    // TODO show user the error here
                                    eprintln!("selected path {} is a directory", other);
                                    action_stack.clear();
                                    user_data.modify_existing = false;
                                    continue;
                                } else {
                                    panic!("something is very wrong");
                                }
                            }
                        };
                        let ast = match rnix::parse(&content).as_result() {
                            Ok(parsed) => parsed,
                            Err(err) => {
                                // TODO show user the error here
                                eprintln!("could not parse {} as a nix file: {}", other, err);
                                action_stack.clear();
                                continue;
                            }
                        };
                        user_data.root = Some(ast.root().inner().unwrap());
                        user_data.modify_existing = false;
                        action_stack.push(UserAction::IntroParsed);
                    }
                    UserAction::RemoveInput => {
                        let dead_node = &user_data
                            .inputs
                            .clone()
                            .unwrap()
                            .get(&other)
                            .unwrap()
                            .parent()
                            .unwrap();
                        let new_root = match kill_node_attribute(dead_node) {
                            Ok(node) => node,
                            Err(err) => {
                                // TODO show user the error here
                                eprintln!("could not remove input: {}", err);
                                action_stack.clear();
                                continue;
                            }
                        };
                        user_data.new_root(new_root);
                        println!("{}", user_data.root.as_ref().unwrap().to_string());
                        // TODO better error handling
                        //root.unwrap():
                        action_stack.push(UserAction::IntroParsed);
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }
}
