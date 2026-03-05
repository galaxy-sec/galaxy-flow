use super::model::PatchAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchView {
    pub action: PatchAction,
    pub file: String,
    pub marker: String,
    pub marker_hits: usize,
    pub changed_lines: usize,
    pub dry_run: bool,
}

impl PatchView {
    pub fn render(&self) -> String {
        format!(
            "action={} file={} marker={} marker_hits={} changed_lines={} dry_run={}",
            self.action.as_str(),
            self.file,
            self.marker,
            self.marker_hits,
            self.changed_lines,
            self.dry_run
        )
    }
}
