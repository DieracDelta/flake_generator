mod parser;
mod user;

use anyhow::anyhow;
use parser::file::{filename_to_node, write_to_node};
use parser::utils::remove_input;
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
                action_stack.push(UserAction::Error(anyhow!(format!(
                    "could not process prompt: {}",
                    err
                ))));
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
                        match filename_to_node(filename, &other) {
                            Err(err_msg) => {
                                action_stack.push(UserAction::Error(err_msg))
                            }
                            Ok(root) => {
                                user_data.filename = Some(filename.to_string());
                                user_data.root = Some(root);
                                action_stack.push(UserAction::IntroParsed);
                            }
                        }
                    }
                    UserAction::RemoveInput => {
                        let inputs = user_data.inputs.as_ref().unwrap();
                        let new_root = remove_input(
                            user_data.root.as_ref().unwrap(),
                            other.0.as_str(),
                            Some(inputs),
                        )
                        .unwrap();
                        user_data.new_root(new_root);
                        write_to_node(&user_data);

                        // TODO add in a "write to file" option at the end instead of writing after every modification
                        action_stack.push(UserAction::IntroParsed);
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }
}
