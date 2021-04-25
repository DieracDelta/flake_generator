use anyhow::anyhow;
use parse_display::{Display, FromStr};
use rust_nix_templater::{options::RustToolchainChannel, *};

use crate::ActionStack;

use super::{SmlStr, UserAction, UserMetadata, UserPrompt};

#[derive(Debug, Clone, PartialEq, Eq, Display, FromStr)]
pub(crate) enum Prompt {
    #[display("generate flake")]
    Generate,
    #[display("set package name ({0})")]
    SetPackageName(SmlStr),
    #[display("set executable name ({0})")]
    SetExecName(SmlStr),
    #[display("set description ({0})")]
    SetDescription(SmlStr),
    #[display("set long description ({0})")]
    SetLongDescription(SmlStr),
    #[display("set toolchain ({0})")]
    SetToolchain(RustToolchainChannel),
    #[display("set license ({0})")]
    SetLicense(SmlStr),
    #[display("set desktop file desktop name ({0})")]
    SetDesktopFileName(SmlStr),
    #[display("set desktop file generic name ({0})")]
    SetDesktopFileGenericName(SmlStr),
    #[display("set desktop file comment ({0})")]
    SetDesktopFileComment(SmlStr),
    #[display("set desktop file categories ({0})")]
    SetDesktopFileCategories(SmlStr),
    #[display("choose icon path ({0})")]
    SetIcon(SmlStr),
    #[display("set systems [{0}]")]
    SetSystems(String),
    #[display("set cachix name ({0})")]
    SetCachixName(SmlStr),
    #[display("set cachix public key ({0})")]
    SetCachixKey(SmlStr),
    #[display("toggle build outputs ({0})")]
    ToggleBuildOutputs(bool),
    #[display("toggle app outputs ({0})")]
    ToggleAppOutputs(bool),
    #[display("toggle library flag ({0})")]
    ToggleLibrary(bool),
    #[display("toggle github actions ({0})")]
    ToggleGithubCi(bool),
    #[display("toggle gitlab CI ({0})")]
    ToggleGitlabCi(bool),
    #[display("{0} channel")]
    ChooseToolchain(RustToolchainChannel),
    #[display("{0} license")]
    ChooseLicense(SmlStr),
}

impl From<Prompt> for UserPrompt {
    fn from(x: Prompt) -> UserPrompt {
        UserPrompt::Rust(x)
    }
}

impl Prompt {
    pub(crate) fn process_prompt(
        &self,
        action_stack: &mut ActionStack,
        user_data: &mut UserMetadata,
    ) {
        let act = match self {
            Prompt::Generate => match run_with_options(user_data.rust_options.clone(), false) {
                Ok(_) => Action::Generated.into(),
                Err(err) => {
                    UserAction::Error(anyhow!(format!("rust-nix-templater failed: {}", err)))
                }
            },
            Prompt::SetCachixKey(_) => Action::SetCachixKey.into(),
            Prompt::SetCachixName(_) => Action::SetCachixName.into(),
            Prompt::SetSystems(_) => Action::SetSystems.into(),
            Prompt::SetExecName(_) => Action::SetExecName.into(),
            Prompt::SetDescription(_) => Action::SetDescription.into(),
            Prompt::SetLongDescription(_) => Action::SetLongDescription.into(),
            Prompt::SetPackageName(_) => Action::SetPackageName.into(),
            Prompt::SetToolchain(_) => Action::SetToolchain.into(),
            Prompt::SetLicense(_) => Action::SetLicense.into(),
            Prompt::SetIcon(_) => Action::SetIcon.into(),
            Prompt::SetDesktopFileCategories(_) => Action::SetDesktopFileCategories.into(),
            Prompt::SetDesktopFileComment(_) => Action::SetDesktopFileComment.into(),
            Prompt::SetDesktopFileGenericName(_) => Action::SetDesktopFileGenericName.into(),
            Prompt::SetDesktopFileName(_) => Action::SetDesktopFileName.into(),
            Prompt::ToggleGithubCi(_) => {
                user_data.rust_options.github_ci = !user_data.rust_options.github_ci;
                return;
            }
            Prompt::ToggleGitlabCi(_) => {
                user_data.rust_options.gitlab_ci = !user_data.rust_options.gitlab_ci;
                return;
            }
            Prompt::ToggleBuildOutputs(_) => {
                user_data.rust_options.disable_build = !user_data.rust_options.disable_build;
                return;
            }
            Prompt::ToggleAppOutputs(_) => {
                user_data.rust_options.disable_app = !user_data.rust_options.disable_app;
                return;
            }
            Prompt::ToggleLibrary(_) => {
                user_data.rust_options.package_lib = !user_data.rust_options.package_lib;
                return;
            }
            Prompt::ChooseToolchain(toolchain) => {
                user_data.rust_options.rust_toolchain_channel = toolchain.clone();
                action_stack.pop();
                return;
            }
            Prompt::ChooseLicense(license) => {
                user_data.rust_options.package_license = Some(license.clone().into());
                action_stack.pop();
                return;
            }
        };
        action_stack.push(act);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) enum Action {
    #[display("Welcome to Rust flake generator.")]
    Intro,
    #[display("Generated flake at ./flake.nix")]
    Generated,
    #[display("Type the package name.")]
    SetPackageName,
    #[display("Type the description.")]
    SetDescription,
    #[display("Type the long description.")]
    SetLongDescription,
    #[display("Type the executable name.")]
    SetExecName,
    #[display("Choose a toolchain.")]
    SetToolchain,
    #[display("Choose or type a license.")]
    SetLicense,
    #[display("Type desktop name.")]
    SetDesktopFileName,
    #[display("Type generic name.")]
    SetDesktopFileGenericName,
    #[display("Type comment.")]
    SetDesktopFileComment,
    #[display("Type categories.")]
    SetDesktopFileCategories,
    #[display("Choose icon path.")]
    SetIcon,
    #[display("Type systems seperated by spaces. Example: x86_64-linux x86_64-darwin")]
    SetSystems,
    #[display("Type your Cachix cache public key.")]
    SetCachixKey,
    #[display("Type your Cachix cache name.")]
    SetCachixName,
}

impl From<Action> for UserAction {
    fn from(x: Action) -> UserAction {
        UserAction::Rust(x)
    }
}

impl Action {
    pub(crate) fn get_prompt_items(&self, user_data: &mut UserMetadata) -> Vec<UserPrompt> {
        let map_or_def =
            |opt: Option<&String>| opt.map_or_else(|| SmlStr::new_inline("not set"), Into::into);
        match self {
            Action::Intro => vec![
                Prompt::SetSystems(
                    user_data
                        .rust_options
                        .package_systems
                        .as_ref()
                        .map_or_else(String::new, |sys| sys.join(" ")),
                )
                .into(),
                Prompt::SetCachixKey(map_or_def(
                    user_data.rust_options.cachix_public_key.as_ref(),
                ))
                .into(),
                Prompt::SetCachixName(map_or_def(user_data.rust_options.cachix_name.as_ref()))
                    .into(),
                Prompt::SetIcon(map_or_def(user_data.rust_options.package_icon.as_ref())).into(),
                Prompt::SetDesktopFileCategories(map_or_def(
                    user_data.rust_options.package_xdg_categories.as_ref(),
                ))
                .into(),
                Prompt::SetDesktopFileComment(map_or_def(
                    user_data.rust_options.package_xdg_comment.as_ref(),
                ))
                .into(),
                Prompt::SetDesktopFileGenericName(map_or_def(
                    user_data.rust_options.package_xdg_generic_name.as_ref(),
                ))
                .into(),
                Prompt::SetDesktopFileName(map_or_def(
                    user_data.rust_options.package_xdg_desktop_name.as_ref(),
                ))
                .into(),
                Prompt::ToggleGithubCi(user_data.rust_options.github_ci).into(),
                Prompt::ToggleGitlabCi(user_data.rust_options.gitlab_ci).into(),
                Prompt::ToggleBuildOutputs(!user_data.rust_options.disable_build).into(),
                Prompt::ToggleAppOutputs(!user_data.rust_options.disable_app).into(),
                Prompt::ToggleLibrary(user_data.rust_options.package_lib).into(),
                Prompt::SetToolchain(user_data.rust_options.rust_toolchain_channel.clone()).into(),
                Prompt::SetLongDescription(map_or_def(
                    user_data.rust_options.package_long_description.as_ref(),
                ))
                .into(),
                Prompt::SetDescription(map_or_def(
                    user_data.rust_options.package_description.as_ref(),
                ))
                .into(),
                Prompt::SetExecName(map_or_def(
                    user_data.rust_options.package_executable.as_ref(),
                ))
                .into(),
                Prompt::SetPackageName(user_data.rust_options.package_name.as_ref().map_or_else(
                    || {
                        SmlStr::new_inline(if std::path::Path::new("./Cargo.toml").exists() {
                            "not set"
                        } else {
                            "not set, required"
                        })
                    },
                    Into::into,
                ))
                .into(),
                Prompt::SetLicense(map_or_def(user_data.rust_options.package_license.as_ref()))
                    .into(),
                Prompt::Generate.into(),
                UserPrompt::Back,
            ],
            Action::Generated => vec![UserPrompt::StartOver],
            Action::SetExecName
            | Action::SetDescription
            | Action::SetLongDescription
            | Action::SetPackageName
            | Action::SetDesktopFileCategories
            | Action::SetDesktopFileComment
            | Action::SetDesktopFileGenericName
            | Action::SetDesktopFileName
            | Action::SetIcon
            | Action::SetSystems
            | Action::SetCachixKey
            | Action::SetCachixName => vec![],
            Action::SetToolchain => vec![
                Prompt::ChooseToolchain(RustToolchainChannel::Nightly).into(),
                Prompt::ChooseToolchain(RustToolchainChannel::Beta).into(),
                Prompt::ChooseToolchain(RustToolchainChannel::Stable).into(),
                UserPrompt::Back,
            ],
            Action::SetLicense => vec![
                Prompt::ChooseLicense(SmlStr::new_inline("MIT")).into(),
                Prompt::ChooseLicense(SmlStr::new_inline("GPLv3")).into(),
                Prompt::ChooseLicense(SmlStr::new_inline("GPLv2")).into(),
                UserPrompt::Back,
            ],
        }
    }

    // Called when `UserPrompt::Other(String)`
    pub(crate) fn process_action(
        &self,
        other: SmlStr,
        action_stack: &mut ActionStack,
        user_data: &mut UserMetadata,
    ) {
        let mut other = other.0.trim().to_string();
        let opt = match self {
            Action::SetCachixKey => &mut user_data.rust_options.cachix_public_key,
            Action::SetCachixName => &mut user_data.rust_options.cachix_name,
            Action::SetPackageName => &mut user_data.rust_options.package_name,
            Action::SetDescription => &mut user_data.rust_options.package_description,
            Action::SetLongDescription => &mut user_data.rust_options.package_long_description,
            Action::SetExecName => &mut user_data.rust_options.package_executable,
            Action::SetLicense => &mut user_data.rust_options.package_license,
            Action::SetDesktopFileCategories => &mut user_data.rust_options.package_xdg_categories,
            Action::SetDesktopFileComment => &mut user_data.rust_options.package_xdg_comment,
            Action::SetDesktopFileGenericName => {
                &mut user_data.rust_options.package_xdg_generic_name
            }
            Action::SetDesktopFileName => &mut user_data.rust_options.package_xdg_desktop_name,
            Action::SetIcon => {
                user_data.rust_options.package_icon = Some(if !other.starts_with("./") {
                    other.insert_str(0, "./");
                    other
                } else {
                    other
                });
                action_stack.pop();
                return;
            }
            Action::SetSystems => {
                user_data.rust_options.package_systems = (!other.is_empty())
                    .then(|| other.split_whitespace().map(str::to_string).collect());
                action_stack.pop();
                return;
            }
            Action::SetToolchain => {
                action_stack.push(UserAction::Error(format!(
                    "{} is not a valid toolchain channel.",
                    other
                )));
                return;
            }
            _ => unreachable!(),
        };
        *opt = (!other.is_empty()).then(|| other);
        action_stack.pop();
    }
}
