use getset::Getters;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatchAction {
    Set,
    CommentLine,
    UncommentLine,
    CommentBlock,
    UncommentBlock,
}

impl PatchAction {
    pub fn as_str(self) -> &'static str {
        match self {
            PatchAction::Set => "set",
            PatchAction::CommentLine => "comment_line",
            PatchAction::UncommentLine => "uncomment_line",
            PatchAction::CommentBlock => "comment_block",
            PatchAction::UncommentBlock => "uncomment_block",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Getters, Builder)]
#[builder(setter(into))]
#[getset(get = "pub")]
pub struct GxPatchFile {
    file: String,
    marker: String,
    action: PatchAction,
    #[builder(default)]
    value: Option<String>,
    #[builder(default = "true")]
    strict: bool,
    #[builder(default = "false")]
    dry_run: bool,
    #[builder(default = "false")]
    backup: bool,
    #[builder(default = "\"#\".to_string()")]
    comment_prefix: String,
}
