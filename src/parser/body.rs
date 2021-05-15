#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Input {
    pub(crate) name: Option<SyntaxStructure>,
    pub(crate) url: Option<SyntaxStructure>,
    pub(crate) is_flake: Option<SyntaxStructure>,
}


