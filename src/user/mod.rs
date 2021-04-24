pub mod rust;

use crate::parser::parser_utils::{get_inputs, NixNode};

use std::{collections::HashMap, io::Cursor, str::FromStr};

use parse_display::{Display, FromStr};
use skim::prelude::*;
use smol_str::SmolStr;

/// `SmlStr` wraps [`SmolStr`] to additionally provide a [`FromStr`] implementation.
/// Can be removed after https://github.com/rust-analyzer/smol_str/issues/31 is fixed.
#[derive(Debug, Default, Clone, PartialEq, Eq, Display)]
pub struct SmlStr(pub SmolStr);

impl SmlStr {
    pub const fn new_inline(s: &str) -> Self {
        Self(SmolStr::new_inline(s))
    }
}

impl From<SmlStr> for String {
    fn from(x: SmlStr) -> String {
        x.0.into()
    }
}

impl From<String> for SmlStr {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<&String> for SmlStr {
    fn from(s: &String) -> Self {
        Self(s.into())
    }
}

impl FromStr for SmlStr {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SmlStr(s.into()))
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct UserMetadata {
    pub(crate) root: Option<NixNode>,
    pub(crate) inputs: Option<HashMap<String, (String, NixNode)>>,
    pub(crate) filename: Option<String>,
    pub(crate) rust_options: rust_nix_templater::Options,
}

impl UserMetadata {
    pub(crate) fn new_root(&mut self, root: NixNode) {
        self.inputs = None;
        self.root = Some(root);
    }

    fn ensure_inputs(&mut self) -> &mut HashMap<String, (String, NixNode)> {
        let root_ref = self.root.as_ref();
        self.inputs
            .get_or_insert_with(|| get_inputs(root_ref.unwrap()))
    }

    pub(crate) fn get_prompt_items(&mut self, action: &UserAction) -> Vec<UserPrompt> {
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
                    .iter()
                    .map(|(attribute, value)| {
                        UserPrompt::from_str(&format!("{}: {}", attribute, value.0)).unwrap()
                    })
                    .chain(std::iter::once(UserPrompt::Back))
                    .collect()
            }
            UserAction::Error(_) => vec![UserPrompt::Back, UserPrompt::StartOver, UserPrompt::Exit],
            x => unimplemented!("prompt not implemented for: {:?}", x),
        }
    }

    pub(crate) fn get_user_prompt(&mut self, a: &UserAction) -> anyhow::Result<UserPrompt> {
        let input = query_user_input(
            a.to_string().lines(),
            self.get_prompt_items(a).into_iter(),
            matches!(
                a,
                UserAction::ModifyExisting | UserAction::Rust(rust::Action::SetIcon)
            ),
        )?;
        Ok(UserPrompt::from_str(&input)
            .expect("could not make prompt; this should be impossible, please file a bug report"))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Display, FromStr)]
pub(crate) enum UserPrompt {
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
    Other(SmlStr),
}

#[derive(Eq, PartialEq, Debug, Clone, Display)]
pub(crate) enum UserAction {
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
pub(crate) enum Lang {
    #[display("rust")]
    Rust,
    #[display("haskell")]
    Haskell,
    #[display("python")]
    Python,
    #[display("javascript")]
    JavaScript,
}

pub(crate) fn query_user_input<'a, PI, II>(
    prompt: PI,
    items: II,
    files: bool,
) -> anyhow::Result<String>
where
    PI: Iterator<Item = &'a str> + DoubleEndedIterator,
    II: Iterator<Item = UserPrompt> + DoubleEndedIterator,
{
    fn agg<I>(x: I) -> (usize, String)
    where
        I: Iterator + DoubleEndedIterator,
        I::Item: AsRef<str>,
    {
        x.rev().fold((0, String::new()), |mut acc, ele| {
            acc.0 += 1;
            acc.1.push('\n');
            acc.1.push_str(ele.as_ref());
            acc
        })
    }

    let (header_len, agg_prompt) = agg(prompt);

    let options = SkimOptionsBuilder::default()
        .header(Some(&agg_prompt))
        .header_lines(header_len)
        .prompt(Some("Provide input: "))
        .inline_info(false)
        .multi(false)
        .build()
        .expect("failed to build skim options: something is very wrong");

    let item_reader = SkimItemReader::default();
    let items =
        (!files).then(|| item_reader.of_bufread(Cursor::new(agg(items.map(|i| i.to_string())).1)));

    let result = Skim::run_with(&options, items).expect("skim failed: something is very wrong");
    Ok(result
        .selected_items
        .get(0)
        .map_or(result.query, |item| item.output().to_string()))
}
