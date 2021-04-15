extern crate skim;
use crate::parser::get_inputs;
use rnix::NixLanguage;
use skim::prelude::*;
use std::io::Cursor;

//#[derive(Eq, PartialEq, Debug, Clone)]
//pub struct UserResult {
//user_selection: String,
//}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct UserMetadata<'a> {
    pub root: Option<rowan::api::SyntaxNode<NixLanguage>>,
    pub inputs: Vec<&'a rowan::api::SyntaxNode<NixLanguage>>,
    pub filename: Option<String>,
    pub modify_existing: bool,
}

impl UserMetadata<'_> {
    pub fn root_ref(&self) -> &rowan::api::SyntaxNode<NixLanguage> {
        self.root.as_ref().unwrap()
    }
}

impl Default for UserMetadata<'_> {
    fn default() -> Self {
        UserMetadata {
            root: None,
            inputs: Vec::new(),
            filename: None,
            modify_existing: false,
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

pub fn get_user_result(a: UserAction, md: &UserMetadata) -> String {
    let prompts = get_prompts(a);
    let prompt_items = get_prompt_items(a, md);
    query_user_input(prompts, prompt_items, a == UserAction::ModifyExisting)
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
        x.into_iter()
            .rev()
            .fold("".to_string(), |acc, ele| -> String {
                format!("{}\n{}", acc, ele)
            })
    };
    let agg_prompt = agg(prompt);
    let options = SkimOptionsBuilder::default()
        .header(Some(&agg_prompt))
        // a lot...
        .header_lines(header_len)
        .algorithm(FuzzyAlgorithm::Clangd)
        .prompt(Some("Provide input:"))
        .inline_info(false)
        .multi(false)
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();
    let items = if files {
        None
    } else {
        Some(item_reader.of_bufread(Cursor::new(agg(items))))
    };
    let result = Skim::run_with(&options, items).unwrap();
    if items_len > 0 || files {
        result.selected_items.get(0).unwrap().output().to_string()
    } else {
        result.query
    }
}

pub fn get_prompts(action: UserAction) -> Vec<String> {
    match action {
        UserAction::IntroParsed => vec!["What would you like to do?".to_string()],
        UserAction::AddDep => vec![
            "Add a dependency to your flake.".to_string(),
            "Please select an package from nixpkgs.".to_string(),
        ],
        UserAction::RemoveDep => vec![
            "Remove a dependency from your flake. ".to_string(),
            "Please select a input to remove.".to_string(),
        ],
        UserAction::AddInput => vec![
            "Add an input to your flake.".to_string(),
            "Please input a flake url and indicate if it's a flake".to_string(),
        ],
        UserAction::IsInputFlake => vec!["Is the input a flake?".to_string()],
        UserAction::RemoveInput => vec!["Please select an input to remove.".to_string()],
        UserAction::GenLib => unimplemented!(),
        UserAction::GenBin(_) => unimplemented!(),
        UserAction::ModifyExisting => vec!["Choose the flake.".to_string()],
        UserAction::CreateNew => unimplemented!(),
        UserAction::Intro => vec![
            "Welcome. Would you like to create a new flake or modify an existing flake?"
                .to_string(),
        ],
        _ => unimplemented!(),
    }
}

pub fn get_prompt_items(action: UserAction, md: &UserMetadata) -> Vec<String> {
    match action {
        UserAction::Intro => vec!["create".to_string(), "modify".to_string()],
        UserAction::IntroParsed => {
            vec!["Delete existing input".to_string(), "Add input".to_string()]
        }
        UserAction::ModifyExisting => vec![],
        UserAction::RemoveInput => get_inputs(md.root_ref()),
        _ => vec![],
    }
}
