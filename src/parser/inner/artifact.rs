use super::super::prelude::*;
use super::common::action_call_args;

use crate::ability::artifact::GxArtifact;
use crate::ability::GxArtifactBuilder;
use crate::parser::domain::gal_keyword;
pub fn gal_artifact(input: &mut &str) -> ModalResult<GxArtifact> {
    let mut down = GxArtifactBuilder::default();
    gal_keyword("gx.artifact", input)?;
    let props = action_call_args.parse_next(input)?;
    for (k, v) in &props {
        if k == "file" {
            down.pkg_file(v.clone());
        }
        if k == "dst_path" {
            down.dst_path(v);
        }
    }
    match down.build() {
        Ok(o) => Ok(o),
        Err(e) => {
            error!("{e}");
            fail.context(wn_desc("gx.artifact")).parse_next(input)
        }
    }
}
