use super::prelude::*;
use crate::components::gxl_extend::ModAddr;
use crate::evaluator::EnvExpress;
use crate::execution::VarSpace;
use crate::model::expect::ShellOption;
use crate::parser::abilities::addr::gal_extern_mod;
use crate::parser::abilities::addr::gal_git_path;
use crate::util::GitTools;
use crate::ExecResult;
use orion_error::ErrorOwe;
use orion_error::ErrorWith;
use orion_error::WithContext;
use std::fs::read_to_string;
use winnow::ascii::line_ending;
use winnow::ascii::till_line_ending;

use winnow::stream::Stream;

#[derive(Debug)]
pub enum DslStatus {
    Extern,
    Code,
    Data,
    End,
}

#[derive(Default, Builder, Getters)]
struct ExternLocal {
    path: String,
}

#[derive(Default, Builder)]
struct ExternGit {
    // root: String,
    url: String,
    tag: String,
    repo: String,
}

impl ExternGit {
    pub fn pull(&self, tools: &GitTools, expect: &ShellOption) -> ExecResult<()> {
        tools.pull_mod(
            self.url.as_str(),
            self.repo.as_str(),
            self.tag.as_str(),
            expect,
        )
    }
    pub fn vendor_path(&self, tools: &GitTools) -> String {
        format!("{}/{}-{}/mods", tools.vendor_root(), self.repo, self.tag)
    }
}

impl ExternLocal {
    pub fn fetch_code(&self, name: &str) -> ExecResult<String> {
        let mut ctx = WithContext::want("load code");
        let ee = EnvExpress::from_env();
        let gxl_full_path = format!("{}/{}.gxl", self.path, name);
        let gxl_full_path = crate::evaluator::Parser::eval(&ee, &gxl_full_path)?;
        ctx.with("gxl", gxl_full_path.as_str());
        let code = read_to_string(gxl_full_path.as_str())
            .owe_rule()
            .with(&ctx)?;
        Ok(code)
    }
}

pub struct ExternParser {}
impl Default for ExternParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternParser {
    pub fn new() -> Self {
        ExternParser {}
    }
    pub fn parse_code(input: &mut &str) -> Result<(String, DslStatus)> {
        let mut out = String::new();

        loop {
            if input.is_empty() {
                break;
            }
            let ck = input.checkpoint();
            if starts_with((multispace0, "extern"), input) {
                input.reset(&ck);
                return Ok((out, DslStatus::Extern));
            }

            // 解析当前行（包括空字符）
            let line = till_line_ending.parse_next(input)?;
            out.push_str(line);

            // 如果输入未结束，跳过换行符
            if !input.is_empty() {
                let end = line_ending(input)?;
                out.push_str(end);
            }
        }

        Ok((out, DslStatus::End))
    }

    pub fn parse_extend_mod(
        cur: &mut &str,
        git: &GitTools,
        shell_opt: &ShellOption,
        vars_space: &VarSpace,
    ) -> ExecResult<(String, DslStatus)> {
        use crate::evaluator::Parser;
        let extern_mods = gal_extern_mod
            .context(wn_desc("<extern-mod>"))
            .parse_next(cur)
            .owe_rule()?;
        let exp = EnvExpress::from_env_mix(vars_space.global().clone());
        let mut git_builder = ExternGitBuilder::default();
        git_builder.tag("".into());
        let local = match extern_mods.addr() {
            ModAddr::Git(git_addr) => {
                let git_url = exp.eval(git_addr.remote())?;
                let cl_git_url = git_url.clone();
                let (_, repo_name) = gal_git_path(&mut git_url.as_str()).owe_rule()?;
                debug!("git url: {cl_git_url}");
                debug!("git repo : {repo_name}",);
                git_builder.url(cl_git_url);
                git_builder.repo(repo_name);
                git_builder.tag(git_addr.channel().into());
                let git_cmd = git_builder.build().owe_rule()?;
                git_cmd.pull(git, shell_opt)?;
                let local_path = git_cmd.vendor_path(git);
                //props.insert("path".into(), local_path);
                ExternLocalBuilder::default()
                    .path(local_path.clone())
                    .build()
                    .unwrap()
            }
            ModAddr::Loc(loc_addr) => ExternLocalBuilder::default()
                .path(exp.eval(loc_addr.path())?)
                .build()
                .unwrap(),
        };
        debug!("mod-local @PATH: {}", local.path());
        let mut out = String::new();
        for mod_name in extern_mods.mods() {
            let mut code = local.fetch_code(mod_name)?;
            code = code.replace("@{PATH}", local.path());
            code = code.replace("@PATH", local.path());
            out += code.as_str();
        }
        Ok((out, DslStatus::Code))
    }
    pub fn extern_parse(
        &self,
        git: &GitTools,
        shell_opt: &ShellOption,
        input: &mut &str,
        vars_space: &VarSpace,
    ) -> ExecResult<(String, bool)> {
        let mut status = DslStatus::Code;
        let mut out = String::new();
        let mut have_extern = false;
        loop {
            if input.is_empty() {
                break;
            }
            match status {
                DslStatus::Code => {
                    let (code, cur_status) = Self::parse_code(input).owe_data()?;
                    out += code.as_str();
                    status = cur_status;
                    continue;
                }
                DslStatus::Extern => {
                    let (code, cur_status) =
                        Self::parse_extend_mod(input, git, shell_opt, vars_space)?;
                    out += code.as_str();
                    status = cur_status;
                    have_extern = true;
                }
                DslStatus::Data => todo!(),
                DslStatus::End => break,
            }
        }
        Ok((out, have_extern))
    }
}

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use super::*;
    #[test]
    fn test_extern_one() {
        let git_t = GitTools::new(false).expect("git tool init fail!");
        let sh_opt = ShellOption::default();
        let parser = ExternParser::new();
        let vars = VarSpace::sys_init().assert();
        let mut data = r#"extern mod ssh { path = "./_gal/mods";}"#;
        let (codes, _have_ext) = parser
            .extern_parse(&git_t, &sh_opt, &mut data, &vars)
            .assert();

        let mut expect = read_to_string("./_gal/mods/ssh.gxl").unwrap();
        expect = expect.replace("@PATH", "./_gal/mods");
        assert_eq!(codes, expect);
    }
    #[test]
    fn test_extern_muti() {
        let vars = VarSpace::sys_init().assert();
        let git_t = GitTools::new(false).expect("git tool init fail!");
        let sh_opt = ShellOption::default();
        let parser = ExternParser::new();
        let mut data = r#"extern mod os,ssh { path = "./_gal/mods";}"#;
        let (codes, _have_ext) = parser
            .extern_parse(&git_t, &sh_opt, &mut data, &vars)
            .assert();
        let mut expect = read_to_string("./_gal/tests/_all.gxl").assert();
        expect = expect.replace("@PATH", "./_gal/mods");
        println!("{codes}",);
        assert_eq!(codes, expect);
    }
}
