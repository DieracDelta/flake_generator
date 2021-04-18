use crate::parser::{self, NixNode};

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    io::Cursor,
    str::FromStr,
};

use skim::prelude::*;

//#[derive(Eq, PartialEq, Debug, Clone)]
//pub struct UserResult {
//user_selection: String,
//}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct UserMetadata {
    pub root: Option<NixNode>,
    pub inputs: Option<HashMap<String, NixNode>>,
    pub filename: Option<String>,
    pub modify_existing: bool,
}

impl UserMetadata {
    pub fn root_ref(&self) -> &NixNode {
        self.root.as_ref().unwrap()
    }

    pub fn new_root(&mut self, root: NixNode) {
        self.inputs = None;
        self.root = Some(root);
    }

    pub fn get_inputs(&mut self) -> HashMap<String, NixNode> {
        if let Some(inputs) = &mut self.inputs {
            inputs.clone()
        } else {
            let updated_inputs = parser::get_inputs(self.root_ref());
            self.inputs = Some(updated_inputs.clone());
            updated_inputs
        }
    }

    pub fn get_prompt_items(&mut self, action: UserAction) -> Vec<UserPrompt> {
        match action {
            UserAction::Intro => vec![UserPrompt::Create, UserPrompt::Modify, UserPrompt::Exit],
            UserAction::IntroParsed => vec![
                UserPrompt::DeleteInput,
                UserPrompt::AddInput,
                UserPrompt::Back,
            ],
            UserAction::ModifyExisting => vec![UserPrompt::Back],
            UserAction::RemoveInput => {
                //check cache
                let mut prompts: Vec<UserPrompt> = self
                    .inputs
                    .as_ref()
                    .map(|inputs| {
                        inputs
                            .keys()
                            .map(|s| UserPrompt::from_str(s).unwrap())
                            .collect()
                    })
                    .unwrap_or_else(|| {
                        self.get_inputs()
                            .keys()
                            .map(|s| UserPrompt::from_str(s).unwrap())
                            .collect()
                    });
                prompts.push(UserPrompt::Back);
                prompts
            }
            x => unimplemented!("prompt not implemented for: {:?}", x),
        }
    }

    pub fn get_user_result(&mut self, a: UserAction) -> String {
        query_user_input(
            a.to_string().lines().map(str::to_string).collect(),
            self.get_prompt_items(a)
                .into_iter()
                .map(|p| p.to_string())
                .collect(),
            a == UserAction::ModifyExisting,
        )
    }
}

impl Default for UserMetadata {
    fn default() -> Self {
        UserMetadata {
            root: None,
            inputs: None,
            filename: None,
            modify_existing: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UserPrompt {
    Back,
    Exit,
    Create,
    Modify,
    DeleteInput,
    AddInput,
    Other(String),
}

impl UserPrompt {
    pub fn as_str(&self) -> &str {
        match self {
            UserPrompt::Back => "back",
            UserPrompt::Exit => "exit",
            UserPrompt::Create => "create",
            UserPrompt::Modify => "modify",
            UserPrompt::DeleteInput => "delete input",
            UserPrompt::AddInput => "add input",
            UserPrompt::Other(o) => &o,
        }
    }
}

impl Display for UserPrompt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for UserPrompt {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == UserPrompt::Create.as_str() {
            Ok(UserPrompt::Create)
        } else if s == UserPrompt::Modify.as_str() {
            Ok(UserPrompt::Modify)
        } else if s == UserPrompt::AddInput.as_str() {
            Ok(UserPrompt::AddInput)
        } else if s == UserPrompt::DeleteInput.as_str() {
            Ok(UserPrompt::DeleteInput)
        } else if s == UserPrompt::Exit.as_str() {
            Ok(UserPrompt::Exit)
        } else if s == UserPrompt::Back.as_str() {
            Ok(UserPrompt::Back)
        } else {
            Ok(UserPrompt::Other(s.into()))
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum UserAction {
    Intro,
    IntroParsed,
    Exit,
    ModifyExisting,
    CreateNew,
    AddDep,
    RemoveDep,
    AddInput,
    RemoveInput,
    GenLib,
    IsInputFlake,
    GenBin(Lang),
}

impl Display for UserAction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            UserAction::IntroParsed => write!(f, "What would you like to do?"),
            UserAction::AddDep => write!(f, "Add a dependency to your flake.\nPlease select an package from nixpkgs."),
            UserAction::RemoveDep => write!(f, "Remove a dependency from your flake.\nPlease select a input to remove."),
            UserAction::AddInput => write!(f, "Add an input to your flake.\nPlease input a flake url and indicate if it's a flake"),
            UserAction::IsInputFlake => write!(f, "Is the input a flake?"),
            UserAction::RemoveInput => write!(f, "Please select an input to remove."),
            UserAction::GenLib => unimplemented!(),
            UserAction::GenBin(_) => unimplemented!(),
            UserAction::ModifyExisting => write!(f, "Choose the flake."),
            UserAction::CreateNew => unimplemented!(),
            UserAction::Intro => write!(f, "Welcome. Would you like to create a new flake or modify an existing flake?"),
            _ => unimplemented!(),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Lang {
    Rust,
    Haskell,
    Python,
    JavaScript,
}

pub fn query_user_input(prompt: Vec<String>, items: Vec<String>, files: bool) -> String {
    let header_len = prompt.len();
    let items_len = items.len();

    let agg = |x: Vec<String>| -> String {
        x.into_iter().rev().fold("".to_string(), |mut acc, ele| {
            acc.push('\n');
            acc.push_str(&ele);
            acc
        })
    };
    let agg_prompt = agg(prompt);

    let options = SkimOptionsBuilder::default()
        .algorithm(FuzzyAlgorithm::Clangd)
        .header(Some(&agg_prompt))
        .header_lines(header_len)
        .prompt(Some("Provide input:"))
        .inline_info(false)
        .multi(false)
        .build()
        .expect("failed to build skim options: something is very wrong");

    let item_reader = SkimItemReader::default();
    let items = (!files).then(|| item_reader.of_bufread(Cursor::new(agg(items))));

    let result = Skim::run_with(&options, items).expect("skim failed: something is very wrong");
    if items_len > 0 || files {
        result.selected_items.get(0).unwrap().output().to_string()
    } else {
        result.query
    }
}
