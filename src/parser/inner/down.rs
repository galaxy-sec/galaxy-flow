use super::super::prelude::*;
use super::common::sentence_body;

use crate::ability::download::GxDownLoad;
use crate::ability::download::GxDownLoadBuilder;
use crate::parser::domain::gal_keyword_alt;
pub fn gal_downlaod(input: &mut &str) -> ModalResult<GxDownLoad> {
    let mut down = GxDownLoadBuilder::default();
    gal_keyword_alt("gx.down", "gx.download", input)?;
    let props = sentence_body.parse_next(input)?;
    for (k, v) in &props {
        if k == "file" {
            down.task_file(v.clone());
        }
        if k == "dst_path" {
            down.dst_path(v);
        }
    }
    match down.build() {
        Ok(o) => Ok(o),
        Err(e) => {
            error!("{}", e);
            fail.context(wn_desc("gx.down")).parse_next(input)
        }
    }
}
