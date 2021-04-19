pub mod rust;

use crate::parser::{self, NixNode};

use std::{collections::HashMap, io::Cursor, str::FromStr};

use anyhow::anyhow;
use parse_display::{Display, FromStr};
use skim::prelude::*;

#[derive(Debug, Default)]
pub struct UserMetadata {
    pub root: Option<NixNode>,
    pub inputs: Option<HashMap<String, NixNode>>,
    pub filename: Option<String>,
    pub rust_options: rust_nix_templater::Options,
}

impl UserMetadata {
    pub fn root_ref(&self) -> &NixNode {
        self.root.as_ref().unwrap()
    }

    pub fn new_root(&mut self, root: NixNode) {
        self.inputs = None;
        self.root = Some(root);
    }

    fn ensure_inputs(&mut self) -> &mut HashMap<String, NixNode> {
        let root_ref = self.root.as_ref();
        self.inputs
            .get_or_insert_with(|| parser::get_inputs(root_ref.unwrap()))
    }

    pub fn get_inputs(&mut self) -> HashMap<String, NixNode> {
        self.ensure_inputs().clone()
    }

    pub fn get_prompt_items(&mut self, action: &UserAction) -> Vec<UserPrompt> {
        match action {
            UserAction::Rust(act) => act.get_prompt_items(self),
            UserAction::Intro => vec![UserPrompt::Create, UserPrompt::Modify, UserPrompt::Exit],
            UserAction::IntroParsed => vec![
                UserPrompt::DeleteInput,
                UserPrompt::AddInput,
                UserPrompt::Back,
            ],
            UserAction::CreateNew => vec![UserPrompt::SelectLang(Lang::Rust), UserPrompt::Back],
            UserAction::ModifyExisting => vec![],
            UserAction::RemoveInput => {
                // check cache
                self.ensure_inputs()
                    .keys()
                    .map(|s| UserPrompt::from_str(s).unwrap())
                    .chain(std::iter::once(UserPrompt::Back))
                    .collect()
            }
            UserAction::Error(_) => vec![UserPrompt::Back, UserPrompt::StartOver, UserPrompt::Exit],
            x => unimplemented!("prompt not implemented for: {:?}", x),
        }
    }

    pub fn get_user_prompt(&mut self, a: &UserAction) -> anyhow::Result<UserPrompt> {
        let input = query_user_input(
            a.to_string().lines().map(str::to_string).collect(),
            self.get_prompt_items(a)
                .into_iter()
                .map(|p| p.to_string())
                .collect(),
            &UserAction::ModifyExisting == a,
        )?;
        Ok(UserPrompt::from_str(&input).unwrap())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Display, FromStr)]
pub enum UserPrompt {
    #[display("start over")]
    StartOver,
    #[display("back")]
    Back,
    #[display("exit")]
    Exit,
    #[display("create")]
    Create,
    #[display("modify")]
    Modify,
    #[display("delete input")]
    DeleteInput,
    #[display("add input")]
    AddInput,
    #[display("{0}")]
    Rust(rust::Prompt),
    #[display("{0}")]
    SelectLang(Lang),
    #[display("{0}")]
    Other(String),
}

#[derive(Eq, PartialEq, Debug, Clone, Display)]
pub enum UserAction {
    #[display("Welcome. Would you like to create a new flake or modify an existing flake?")]
    Intro,
    #[display("What would you like to do?")]
    IntroParsed,
    #[display("Choose the flake.")]
    ModifyExisting,
    #[display("Choose a flake generator.")]
    CreateNew,
    #[display("Add a dependency to your flake.\nPlease select an package from nixpkgs.")]
    AddDep,
    #[display("Remove a dependency from your flake.\nPlease select a input to remove.")]
    RemoveDep,
    #[display(
        "Add an input to your flake.\nPlease input a flake url and indicate if it's a flake"
    )]
    AddInput,
    #[display("Please select an input to remove.")]
    RemoveInput,
    #[display("Is the input a flake?")]
    IsInputFlake,
    #[display("Encountered an error: {0}")]
    Error(String),
    #[display("{0}")]
    Rust(rust::Action),
}

#[derive(Eq, PartialEq, Debug, Copy, Clone, Display, FromStr)]
pub enum Lang {
    #[display("rust")]
    Rust,
    #[display("haskell")]
    Haskell,
    #[display("python")]
    Python,
    #[display("javascript")]
    JavaScript,
}

pub fn query_user_input(
    prompt: Vec<String>,
    items: Vec<String>,
    files: bool,
) -> anyhow::Result<String> {
    let header_len = prompt.len();
    let items_len = items.len();

    let agg = |x: Vec<String>| -> String {
        x.into_iter().rev().fold(String::new(), |mut acc, ele| {
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
    Ok(if items_len > 0 || files {
        result
            .selected_items
            .get(0)
            .ok_or_else(|| anyhow!("no chosen item"))?
            .output()
            .to_string()
    } else {
        result.query
    })
}
