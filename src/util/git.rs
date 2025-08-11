use orion_error::ErrorOwe;
use orion_variate::addr::Address;
use orion_variate::addr::GitRepository;
use orion_variate::types::ResourceDownloader;
use orion_variate::types::UpdateUnit;
use orion_variate::update::DownloadOptions;
use orion_variate::vars::EnvDict;

use crate::evaluator::EnvExpress;
use crate::evaluator::VarParser;
use crate::ExecResult;

use std::io::prelude::*;
use std::path::PathBuf;

use super::accessor::build_accessor;
const RG_ROOT: &str = "${HOME}/.galaxy";
const VENDOR_ROOT: &str = "${HOME}/.galaxy/vendor";
#[derive(Default, Getters)]
pub struct GitTools {
    gxl_root: String,
    vendor_root: String,
    force: bool,
    exp_engine: EnvExpress,
}
impl GitTools {
    pub fn new(force: bool) -> ExecResult<Self> {
        let ee = EnvExpress::from_env();
        let rg_root = ee.eval(RG_ROOT)?;
        let vendor_root = ee.eval(VENDOR_ROOT)?;
        Ok(GitTools {
            force,
            gxl_root: rg_root,
            vendor_root,
            exp_engine: ee,
        })
    }
    pub async fn update_mod(
        &self,
        addr: GitRepository,
        options: &DownloadOptions,
    ) -> ExecResult<UpdateUnit> {
        build_accessor(&EnvDict::default())
            .download_to_local(
                &Address::from(addr),
                &PathBuf::from(self.vendor_root()),
                options,
            )
            .await
            .owe_res()
    }
    pub fn vendor_path(&self, repo: &str, tag: &str) -> String {
        format!("{}/{repo}-{tag}/mods", self.vendor_root())
    }
}
