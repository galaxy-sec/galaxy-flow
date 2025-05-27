use super::super::prelude::*;
use super::common::sentence_body;

use crate::ability::{GxDownLoad, GxDownLoadBuilder};
use crate::ability::{GxUpLoad, GxUpLoadBuilder};
use crate::parser::domain::gal_keyword;
pub fn gal_download(input: &mut &str) -> ModalResult<GxDownLoad> {
    let mut down = GxDownLoadBuilder::default();
    gal_keyword("gx.download", input)?;
    let props = sentence_body.parse_next(input)?;
    for (k, v) in &props {
        if k == "url" {
            down.svc_url(v.clone());
        } else if k == "local_file" {
            down.local_file(v.clone());
        } else if k == "username" {
            down.username(v.clone());
        } else if k == "password" {
            down.password(v.clone());
        }
    }
    match down.build() {
        Ok(o) => Ok(o),
        Err(e) => {
            error!("{}", e);
            fail.context(wn_desc("gx.download")).parse_next(input)
        }
    }
}

pub fn gal_upload(input: &mut &str) -> ModalResult<GxUpLoad> {
    let mut down = GxUpLoadBuilder::default();
    gal_keyword("gx.upload", input)?;
    let props = sentence_body.parse_next(input)?;
    for (k, v) in &props {
        if k == "url" {
            down.svc_url(v.clone());
        } else if k == "local_file" {
            down.local_file(v.clone());
        } else if k == "username" {
            down.username(v.clone());
        } else if k == "password" {
            down.password(v.clone());
        }
    }
    match down.build() {
        Ok(o) => Ok(o),
        Err(e) => {
            error!("{}", e);
            fail.context(wn_desc("gx.upload")).parse_next(input)
        }
    }
}
