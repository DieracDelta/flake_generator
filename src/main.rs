mod parser;
mod user;

use std::fs;

use parser::kill_node_attribute;
use rnix::types::*;
use user::*;

struct ActionStack {
    inner: Vec<UserAction>,
}

impl ActionStack {
    const START_ACTION: UserAction = UserAction::Intro;

    fn new() -> Self {
        Self { inner: Vec::new() }
    }

    fn push(&mut self, action: UserAction) {
        self.inner.push(action)
    }

    fn pop(&mut self) -> UserAction {
        self.inner.pop().unwrap_or(Self::START_ACTION)
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    fn current(&self) -> &UserAction {
        self.inner.last().unwrap_or(&Self::START_ACTION)
    }
}

fn main() {
    let mut user_data = UserMetadata::default();
    let mut action_stack = ActionStack::new();

    loop {
        let cur_action = action_stack.current();
        let user_selection = match user_data.get_user_prompt(cur_action) {
            Ok(prompt) => prompt,
            Err(err) => {
                action_stack.push(UserAction::Error(format!(
                    "could not process prompt: {}",
                    err
                )));
                continue;
            }
        };

        match user_selection {
            UserPrompt::Back => {
                action_stack.pop();
            }
            UserPrompt::Exit => break,
            UserPrompt::StartOver => action_stack.clear(),
            UserPrompt::Create => action_stack.push(UserAction::CreateNew),
            UserPrompt::Modify => action_stack.push(UserAction::ModifyExisting),
            UserPrompt::DeleteInput => action_stack.push(UserAction::RemoveInput),
            UserPrompt::AddInput => action_stack.push(UserAction::AddInput),
            UserPrompt::SelectLang(lang) => match lang {
                Lang::Rust => action_stack.push(UserAction::Rust(user::rust::Action::Intro)),
                lang => todo!("lang {}", lang),
            },
            UserPrompt::Rust(prompt) => {
                let action = prompt.get_action(&mut user_data, cur_action);
                action_stack.push(action);
            }
            UserPrompt::Other(other) => {
                match cur_action {
                    UserAction::ModifyExisting => {
                        let content = match fs::read_to_string(&other) {
                            Ok(content) => content,
                            Err(err) => {
                                const IS_DIRECTORY_ERRNO: i32 = 21;
                                let err_msg = if let Some(IS_DIRECTORY_ERRNO) = err.raw_os_error() {
                                    format!("selected path {} is a directory", other)
                                } else if err.kind() == std::io::ErrorKind::InvalidData {
                                    format!(
                                        "selected path {} does not contain valid UTF-8 data",
                                        other
                                    )
                                } else {
                                    format!("something is very wrong: {}", err)
                                };
                                action_stack.push(UserAction::Error(err_msg));
                                continue;
                            }
                        };
                        let ast = match rnix::parse(&content).as_result() {
                            Ok(parsed) => parsed,
                            Err(err) => {
                                action_stack.push(UserAction::Error(format!(
                                    "could not parse {} as a nix file: {}",
                                    other, err
                                )));
                                continue;
                            }
                        };
                        user_data.root = Some(ast.root().inner().unwrap());
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
                                action_stack.push(UserAction::Error(format!(
                                    "could not remove input: {}",
                                    err
                                )));
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
