mod parser;
mod user;

use crate::parser::input_utils::SyntaxStructure;
use anyhow::anyhow;
use nixpkgs_fmt::reformat_node;
use parser::file::{filename_to_node, write_to_node};
use parser::input_utils::{merge_attr_sets, Input};
use parser::utils::{get_node_idx, remove_input, search_for_attr, NixNode};
use rowan::GreenNode;
use rowan::NodeOrToken;
use rowan::SyntaxNode;
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

    fn push_seq(&mut self, actions: impl IntoIterator<Item = UserAction>) {
        actions
            .into_iter()
            .for_each(|action| self.inner.push(action))
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
            UserPrompt::AddInput => action_stack.push_seq(vec![
                UserAction::IntroParsed,
                UserAction::ConfirmWrite,
                UserAction::ConfirmInputCorrect,
                UserAction::QueryInputName,
                UserAction::QueryInputUrl,
                UserAction::IsInputFlake,
            ]),
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
                    UserAction::QueryInputUrl => {
                        let mut i = user_data.new_input.unwrap_or_default();
                        i.url = Some(SyntaxStructure::StringLiteral(other));
                        user_data.new_input = Some(i);
                        action_stack.pop();
                    }
                    UserAction::QueryInputName => {
                        let mut i = user_data.new_input.unwrap_or_default();
                        i.name = Some(SyntaxStructure::StringLiteral(other));
                        user_data.new_input = Some(i);
                        action_stack.pop();
                    }
                    UserAction::ModifyExisting => {
                        let filename = other.0.as_str();
                        match filename_to_node(filename, &other) {
                            Err(err_msg) => action_stack.push(UserAction::Error(err_msg)),
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
                        action_stack.pop();

                        // TODO add in a "write to file" option at the end instead of writing after every modification
                        action_stack.push(UserAction::IntroParsed);
                    }
                    _ => unimplemented!(),
                }
            }
            UserPrompt::Bool(b) => match cur_action {
                UserAction::IsInputFlake => {
                    action_stack.pop();
                    let mut i = user_data
                        .new_input
                        .as_ref()
                        .and_then(|x: &Input| Some(x.clone()))
                        .unwrap_or_default();
                    i.is_flake = Some(SyntaxStructure::Bool(b));
                    user_data.new_input = Some(i);
                }
                UserAction::ConfirmInputCorrect => {
                    let root = user_data.root.as_ref().unwrap();
                    let (inputs, _, _) = search_for_attr("inputs", 1, root, None)[0].clone();
                    let new_input: GreenNode = user_data
                        .new_input
                        .as_ref()
                        .and_then(|x| Some(x.clone()))
                        .unwrap()
                        .into();
                    let augmented_input = merge_attr_sets(inputs.green().to_owned(), new_input);
                    println!("aug: {:?}", augmented_input.to_string());
                    let idx = get_node_idx(&inputs).unwrap();
                    let parent = inputs.parent().unwrap();
                    let new_root = inputs
                        .parent()
                        .unwrap()
                        .green()
                        .to_owned()
                        .replace_child(idx, NodeOrToken::Node(augmented_input));
                    let mut new_root_wrapped: NixNode =
                        SyntaxNode::new_root(parent.replace_with(new_root));
                    while let Some(parent) = new_root_wrapped.parent() {
                        new_root_wrapped = parent;
                    }
                    action_stack.pop();
                    user_data.new_root(reformat_node(&new_root_wrapped))
                }
                UserAction::ConfirmWrite => {
                    action_stack.pop();
                    if b {
                        write_to_node(&user_data)
                    }
                }
                _ => unimplemented!("bool not implemented in this case {}", cur_action),
            },
        }
    }
}
