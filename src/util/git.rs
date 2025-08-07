use orion_error::ErrorOwe;
use orion_variate::addr::Address;
use orion_variate::addr::GitRepository;
use orion_variate::types::ResourceDownloader;
use orion_variate::types::UpdateUnit;
use orion_variate::update::DownloadOptions;
use orion_variate::vars::EnvDict;

use crate::err::*;
use crate::evaluator::*;
use crate::expect::{LogicScope, ShellOption};
use crate::gxl_sh;
use crate::var::VarDict;
use crate::ExecReason;
use crate::ExecResult;
use std::fs;

use std::fs::File;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
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

    pub fn check_run(&self) -> ExecResult<()> {
        self.build_check_shell()?;
        let cmd = format!("{}/{}", self.gxl_root, "git_check.sh");
        debug!(target:"sys", "cmd:{cmd}", );
        let sh_opt = ShellOption {
            quiet: true,
            inner_print: true,
            ..Default::default()
        };
        gxl_sh!(
            LogicScope::Outer,
            "cmd",
            "echo $PATH",
            &sh_opt,
            &self.exp_engine,
            &VarDict::default()
        )?;
        gxl_sh!(
            LogicScope::Outer,
            "cmd",
            "pwd",
            &sh_opt,
            &self.exp_engine,
            &VarDict::default()
        )?;
        gxl_sh!(
            LogicScope::Outer,
            "cmd",
            "ls -l ${HOME}/.galaxy",
            &sh_opt,
            &self.exp_engine,
            &VarDict::default()
        )?;
        gxl_sh!(
            LogicScope::Outer,
            "cmd",
            &cmd,
            &sh_opt,
            &self.exp_engine,
            &VarDict::default()
        )?;
        Ok(())
    }
    fn build_check_shell(&self) -> NER {
        let sh_name = "git_check.sh";
        let sh_path = format!("{}/{}", self.gxl_root, sh_name);
        let shell = include_str!("git_check.sh");
        build_shell(self.gxl_root().as_str(), sh_name, shell, sh_path.as_str())
    }
}

pub fn build_shell(sh_root: &str, sh_name: &str, sh_code: &str, sh_path: &str) -> ExecResult<()> {
    //let sh_path = format!("{}/{}", self.rg_root, SH_NAME);
    if std::path::Path::new(sh_path).exists() {
        return Ok(());
    }
    warn!(target: "sys","will create {sh_name} to {sh_path}", );

    std::fs::create_dir_all(sh_root).owe_res()?;
    let mut file = File::create(sh_path).owe_res()?;
    file.write_all(sh_code.to_string().as_bytes()).owe_res()?;
    let metadata = file.metadata().owe_sys()?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions).owe_res()?;
    if !std::path::Path::new(sh_path).exists() {
        return Err(ExecReason::Depend(format!("create shell fail: {sh_path}",)).into());
    }
    if let Ok(metadata) = fs::metadata(sh_path) {
        if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
            info!(target: "sys","git shell is ready! exists and execute:{sh_path}", );
            return Ok(());
        }
    }
    Err(ExecReason::Depend(format!("shell is bad: {sh_path}",)).into())
}
