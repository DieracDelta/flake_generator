use parse_display::{Display, FromStr};
use rust_nix_templater::*;

use super::{UserAction, UserMetadata, UserPrompt};

#[derive(Debug, Clone, PartialEq, Eq, Display, FromStr)]
pub enum Prompt {
    #[display("generate flake")]
    Generate,
}

impl Prompt {
    pub fn get_action(&self, user_data: &mut UserMetadata, cur_action: &UserAction) -> UserAction {
        match self {
            Prompt::Generate => match run_with_options(user_data.rust_options.clone(), false) {
                Ok(_) => UserAction::Rust(Action::Generated),
                Err(err) => UserAction::Error(format!("rust-nix-templater failed: {}", err)),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display, FromStr)]
pub enum Action {
    #[display("Welcome to Rust flake generator.")]
    Intro,
    #[display("Generated flake at ./flake.nix")]
    Generated,
}

impl Action {
    pub fn get_prompt_items(&self, user_data: &mut UserMetadata) -> Vec<UserPrompt> {
        match self {
            Action::Intro => vec![UserPrompt::Rust(Prompt::Generate), UserPrompt::Back],
            Action::Generated => vec![UserPrompt::StartOver],
        }
    }
}
