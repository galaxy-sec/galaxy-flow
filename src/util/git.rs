use orion_error::ErrorOwe;

use crate::err::*;
use crate::evaluator::*;
use crate::expect::{LogicScope, ShellOption};
use crate::gxl_sh;
use crate::ExecReason;
use crate::ExecResult;
use std::fs;

use std::fs::File;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
const SH_NAME: &str = "remote_git.sh";
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
    pub fn pull_mod(&self, url: &str, repo: &str, tag: &str, opt: &ShellOption) -> ExecResult<()> {
        self.build_remote_git()?;
        let update = if self.force { "true" } else { "false" };
        let cmd = format!(
            "{}/{} {} {}-{} {} {} {}",
            self.gxl_root, SH_NAME, url, repo, tag, tag, update, self.vendor_root
        );

        debug!(target:"sys/mod", "mod update cmd:{cmd}" );
        gxl_sh!(
            LogicScope::Inner,
            "cmd:pull-mod",
            &cmd,
            opt,
            &self.exp_engine
        )?;
        Ok(())
    }
    pub fn pull_init(&self, url: &str, repo: &str, tag: &str, opt: &ShellOption) -> NER {
        self.build_remote_git()?;
        let update = if self.force { "true" } else { "false" };
        let cmd = format!(
            "{}/{} {} {} {} {} {}",
            self.gxl_root, SH_NAME, url, repo, tag, update, self.gxl_root
        );

        debug!(target:"sys/mod", "mod update cmd:{cmd}", );
        gxl_sh!(LogicScope::Inner, "cmd:init", &cmd, opt, &self.exp_engine)?;
        Ok(())
    }
    fn build_remote_git(&self) -> NER {
        let sh_path = format!("{}/{}", self.gxl_root, SH_NAME);
        let shell = include_str!("remote_git.sh");
        build_shell(
            self.gxl_root().as_str(),
            "remote_git",
            shell,
            sh_path.as_str(),
        )
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
            &self.exp_engine
        )?;
        gxl_sh!(LogicScope::Outer, "cmd", "pwd", &sh_opt, &self.exp_engine)?;
        gxl_sh!(
            LogicScope::Outer,
            "cmd",
            "ls -l ${HOME}/.galaxy",
            &sh_opt,
            &self.exp_engine
        )?;
        gxl_sh!(LogicScope::Outer, "cmd", &cmd, &sh_opt, &self.exp_engine)?;
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
