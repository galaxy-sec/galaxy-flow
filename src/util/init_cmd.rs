use super::*;
use crate::expect::LogicScope;
use crate::{err::*, expect::ShellOption};
use crate::{gxl_sh, ExecReason, ExecResult};
use std::path::Path;

#[derive(Default, Builder)]
pub struct ModRepo {
    repo_url: String,
    repo_name: String,
    tag: String,
}

impl ModRepo {
    pub fn new(repo_url: &str, tag: &str) -> ExecResult<Self> {
        let uri = repo_url.split("://").nth(1).unwrap_or("");
        let path = Path::new(uri);
        if let Some(Some(repo_name)) = path.file_name().map(|x| x.to_str()) {
            return Ok(ModRepo {
                repo_url: repo_url.into(),
                tag: tag.to_string(),
                repo_name: repo_name.into(),
            });
        }
        Err(ExecReason::Args(format!("repo url :{repo_url} error")).into())
    }
    pub fn pull(&self, tools: &GitTools, opt: &ShellOption) -> NER {
        if let Err(e) = tools.pull_init(
            self.repo_url.as_str(),
            self.repo_name.as_str(),
            self.tag.as_str(),
            opt,
        ) {
            return Err(ExecReason::Depend(format!("git pull fail: {}", e)).into());
        }
        Ok(())
    }
}

pub fn init_cmd(
    repo: ModRepo,
    dst: &str,
    force: bool,
    tpl_name: &str,
    sh_opt: &ShellOption,
) -> ExecResult<()> {
    let tools = GitTools::new(force)?;
    repo.pull(&tools, sh_opt)?;
    let cmd = format!(
        "export DST_PATH={} ; gx -f {}/{}/_gal/work.gxl  {} ",
        dst,
        tools.gxl_root(),
        repo.repo_name,
        tpl_name,
    );
    gxl_sh!(
        LogicScope::Inner,
        "cmd:init",
        &cmd,
        &sh_opt,
        tools.exp_engine()
    )?;
    println!("init success!");
    Ok(())
}
