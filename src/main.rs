mod parser;
mod user;

use std::fs;
use std::io::Write;

use parser::parser_utils::{get_attr, kill_node_attribute, remove_input_from_output_fn};
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
            UserPrompt::StartOver => {
                user_data = UserMetadata::default();
                action_stack.clear();
            }
            UserPrompt::Create => action_stack.push(UserAction::CreateNew),
            UserPrompt::Modify => action_stack.push(UserAction::ModifyExisting),
            UserPrompt::DeleteInput => action_stack.push(UserAction::RemoveInput),
            UserPrompt::AddInput => action_stack.push(UserAction::AddInput),
            UserPrompt::SelectLang(lang) => match lang {
                Lang::Rust => action_stack.push(UserAction::Rust(user::rust::Action::Intro)),
                lang => todo!("lang {}", lang),
            },
            UserPrompt::Rust(prompt) => {
                prompt.process_prompt(&mut action_stack, &mut user_data);
            }
            UserPrompt::Other(other) => {
                match cur_action {
                    UserAction::Rust(action) => {
                        action
                            .clone()
                            .process_action(other, &mut action_stack, &mut user_data)
                    }
                    UserAction::ModifyExisting => {
                        let filename = other.0.as_str();
                        let content = match fs::read_to_string(filename) {
                            Ok(content) => {
                                // TODO setter
                                user_data.filename = Some(filename.to_string());
                                content
                            }
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
                        let dead_node_path = other.0.as_str();
                        let inputs = user_data.inputs.clone().unwrap();
                        let (dead_node_name, dead_node) = inputs.get(dead_node_path).unwrap();
                        let new_root = match kill_node_attribute(&dead_node.parent().unwrap(), 1) {
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
                        let input_name = get_attr(1, dead_node_name).unwrap();

                        user_data.new_root(
                            remove_input_from_output_fn(
                                &mut (user_data.root.clone().unwrap()),
                                &input_name,
                            )
                            .unwrap(),
                        );
                        // TODO separate out into function
                        let stringified = user_data.root.as_ref().unwrap().to_string();
                        let mut file = fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(user_data.clone().filename.unwrap())
                            .unwrap();
                        file.write(stringified.as_bytes()).unwrap();

                        // TODO better error handling
                        // TODO separate inputs out into a struct
                        // TODO add in a "write to file" option at the end instead of
                        // intermittently
                        //root.unwrap():
                        action_stack.push(UserAction::IntroParsed);
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }
}
