use crate::components::gxl_spc::GxlSpace;
use crate::execution::VarSpace;
use crate::parser::abilities::ignore_comment;
use crate::parser::externs::ExternGit;
use crate::parser::externs::ExternParser;
use crate::parser::stc_spc::gal_stc_spc;
use crate::parser::stc_spc::WinnowErrorEx;

use std::fs;
use std::fs::read_to_string;
use std::path::PathBuf;

use crate::err::*;

use crate::ability::version::Version;
use once_cell::sync::OnceCell;
use orion_error::ErrorConv;
use orion_error::ErrorOwe;
use orion_error::ErrorWith;
use orion_error::WithContext;
use orion_variate::addr::GitAddr;
use orion_variate::addr::LocalAddr;
use orion_variate::types::LocalUpdate;
use orion_variate::update::UpdateOptions;
use orion_variate::update::UpdateScope;
use orion_variate::vars::ValueDict;

static CODE_INSTANCE: OnceCell<String> = OnceCell::new();
pub fn get_parse_code() -> &'static str {
    if let Some(code) = CODE_INSTANCE.get() {
        return code;
    }
    ""
}

#[derive(Getters)]
pub struct GxLoader {
    gal_ver: Version,
}
impl Default for GxLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl GxLoader {
    pub fn new() -> GxLoader {
        GxLoader {
            gal_ver: Version::new(2, 0, 0, None),
        }
    }
    pub async fn parse_file(
        &mut self,
        conf: &str,
        update: bool,
        vars_space: &VarSpace,
    ) -> RunResult<GxlSpace> {
        info!(target:"parse", "parse file: {conf}" );
        let mut wc = WithContext::want("parse gxl file");
        wc.with("conf", conf);
        let code = read_to_string(conf).owe_conf().with(&wc)?;
        self.parse_code(&code, update, vars_space).await
    }
    pub async fn parse_code(
        &self,
        code: &str,
        update: bool,
        vars_space: &VarSpace,
    ) -> RunResult<GxlSpace> {
        let e_parser = ExternParser::new();

        let up_options = if update {
            UpdateOptions::new(UpdateScope::InHost, ValueDict::default())
        } else {
            UpdateOptions::new(UpdateScope::InElm, ValueDict::default())
        };
        let mut target_code = code.to_string();

        loop {
            let mut target_code_str = target_code.as_str();
            let (code, have) = e_parser
                .extern_parse(&up_options, &mut target_code_str, vars_space)
                .await
                .with(("code", err_code_prompt(target_code_str)))
                .err_conv()?;

            target_code_str = code.as_str();
            target_code = ignore_comment(&mut target_code_str)
                .owe(RunReason::Gxl("comment parse".into()))
                .with(err_code_prompt(target_code_str))?;
            if !have {
                break;
            }
        }
        info!(target: "parse","code len: {}", target_code.len());
        fs::write("./.run.gxl", target_code.as_str()).owe_res()?;
        let mut code = target_code.as_str();
        let gxl_space = gal_stc_spc(&mut code)
            .map_err(WinnowErrorEx::from)
            .owe(RunReason::Gxl("gxl error!".into()))
            .position(err_code_prompt(code))
            .want("parse ./.run.gxl file")?;
        Ok(gxl_space)
    }

    pub async fn init(&self, addr: GitAddr, tpl: &str) -> RunResult<()> {
        let up_options = UpdateOptions::new(UpdateScope::InHost, ValueDict::default());
        let local_git = ExternGit::pull(addr, &up_options).await.owe_res()?;
        let vender_path = format!("{}/tpl/{tpl}", local_git.position().display());
        let init_path = PathBuf::from("./_gal");
        if init_path.exists() {
            println!("init dir exists! ({})", init_path.display());
        } else {
            std::fs::create_dir(&init_path).owe_res()?;
            LocalAddr::from(vender_path)
                .update_local(&init_path, &up_options)
                .await
                .owe_res()?;
        }
        Ok(())
    }
}

pub fn err_code_prompt(code: &str) -> String {
    let take_len = if code.len() > 200 { 200 } else { code.len() };
    if let Some((left, _right)) = code.split_at_checked(take_len) {
        return format!("{left}...");
    }
    "".to_string()
}

#[cfg(test)]
mod tests {

    use crate::{execution::VarSpace, infra::once_init_log, types::AnyResult};

    use super::GxLoader;

    #[tokio::test]
    async fn test_parse_file() -> AnyResult<()> {
        //log_init(&LogConf::alpha()).assert();
        once_init_log();
        let mut loader = GxLoader::default();
        let conf = "./_gal/work.gxl";
        let vars = VarSpace::sys_init()?;
        let spc = loader.parse_file(conf, false, &vars).await?.assemble()?;
        info!("test begin");
        spc.show()?;
        println!("mods:{}", spc.len());
        assert!(spc.len() > 1);
        spc.exec(
            ["default".into()].to_vec(),
            ["conf".into()].to_vec(),
            Some(true),
            false,
            VarSpace::sys_init()?,
        )
        .await?;
        Ok(())
    }
}
