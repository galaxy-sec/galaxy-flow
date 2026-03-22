use crate::components::gxl_spc::GxlSpace;
use crate::execution::VarSpace;
use crate::parser::abilities::ignore_comment;
use crate::parser::externs::ExternGit;
use crate::parser::externs::ExternParser;
use crate::parser::stc_spc::WinnowErrorEx;
use crate::parser::stc_spc::gal_stc_spc;
use crate::util::accessor::build_accessor;

use std::fs;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

use crate::err::*;

use crate::ability::version::Version;
use once_cell::sync::OnceCell;
use orion_accessor::addr::Address;
use orion_accessor::addr::GitRepository;
use orion_accessor::addr::LocalPath;
use orion_accessor::types::ResourceDownloader;
use orion_accessor::update::DownloadOptions;
use orion_accessor::update::UpdateScope;
use orion_error::ContextRecord;
use orion_error::ErrorConv;
use orion_error::ErrorOwe;
use orion_error::ErrorOweBase;
use orion_error::ErrorWith;
use orion_error::ToStructError;
use orion_error::WithContext;
use orion_variate::vars::EnvDict;
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
        &self,
        conf: &str,
        update: bool,
        vars_space: &VarSpace,
    ) -> RunResult<GxlSpace> {
        info!(target:"parse", "parse file: {conf}" );
        let mut wc = WithContext::want("parse gxl file");
        wc.record("conf", conf);
        let code = read_to_string(conf).owe_conf().with(&wc)?;
        let file_path = Path::new(conf);
        let file_exist_path = file_path.parent();
        self.parse_code(&code, update, vars_space, file_exist_path)
            .await
    }
    pub async fn parse_code(
        &self,
        code: &str,
        update: bool,
        vars_space: &VarSpace,
        file_exist_path: Option<&Path>,
    ) -> RunResult<GxlSpace> {
        let e_parser = ExternParser::new();

        let up_options = if update {
            DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default())
        } else {
            DownloadOptions::new(UpdateScope::None, ValueDict::default())
        };
        let mut target_code = code.to_string();

        loop {
            let mut target_code_str = target_code.as_str();
            let (code, have) = e_parser
                .extern_parse(
                    &up_options,
                    &mut target_code_str,
                    vars_space,
                    file_exist_path,
                )
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

    pub async fn init_from_git(&self, addr: GitRepository) -> RunResult<()> {
        let init_path = PathBuf::from("./_gal");
        if init_path.exists() {
            return Err(RunReason::Args("_gal already exists".into())
                .to_err()
                .with_detail(format!("path: {}", init_path.display())));
        }

        let up_options = DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default());
        let local_git = ExternGit::pull(addr.clone(), &up_options).await.owe_res()?;
        let src_path = local_git.position();

        // Create _gal only after git pull succeeds
        std::fs::create_dir(&init_path).owe_res()?;

        let accessor = build_accessor(&EnvDict::default());
        let result = accessor
            .download_to_local(
                &Address::from(LocalPath::from(src_path.to_string_lossy().as_ref())),
                &init_path,
                &up_options,
            )
            .await;

        match result {
            Ok(_) => {
                eprintln!("project initialized from git to ./_gal/");
                Ok(())
            }
            Err(e) => {
                // Clean up empty _gal directory on failure
                let _ = std::fs::remove_dir(&init_path);
                Err(RunReason::Exec(format!("copy to _gal failed: {}", e)).to_err())
            }
        }
    }

    pub async fn init_from_local(&self, src: &str) -> RunResult<()> {
        let src_path = PathBuf::from(src);
        if !src_path.exists() {
            return Err(RunReason::Args("template path not found".into())
                .to_err()
                .with_detail(format!("path: {}", src_path.display())));
        }

        let init_path = PathBuf::from("./_gal");
        if init_path.exists() {
            return Err(RunReason::Args("_gal already exists".into())
                .to_err()
                .with_detail(format!("path: {}", init_path.display())));
        }

        // Create _gal only after source validation
        std::fs::create_dir(&init_path).owe_res()?;

        let up_options = DownloadOptions::new(UpdateScope::None, ValueDict::default());
        let accessor = build_accessor(&EnvDict::default());
        let result = accessor
            .download_to_local(
                &Address::from(LocalPath::from(src)),
                &init_path,
                &up_options,
            )
            .await;

        match result {
            Ok(_) => {
                eprintln!("project initialized from {} to ./_gal/", src);
                Ok(())
            }
            Err(e) => {
                // Clean up empty _gal directory on failure
                let _ = std::fs::remove_dir(&init_path);
                Err(RunReason::Exec(format!("copy to _gal failed: {}", e)).to_err())
            }
        }
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

    use crate::{cmd::GxlCmd, execution::VarSpace, infra::once_init_log, types::AnyResult};

    use super::GxLoader;

    #[tokio::test]
    async fn test_parse_file() -> AnyResult<()> {
        //log_init(&LogConf::alpha()).assert();
        once_init_log();
        let loader = GxLoader::default();
        let conf = "./_gal/work.gxl";
        let vars = VarSpace::sys_init()?;
        let spc = loader.parse_file(conf, false, &vars).await?.assemble()?;
        info!("test begin");
        spc.show()?;
        println!("mods:{}", spc.len());
        assert!(spc.len() > 1);
        spc.exec(
            GxlCmd::default()
                .with_env("default".into())
                .with_flows("conf".into()),
            VarSpace::sys_init()?,
            None,
        )
        .await?;
        Ok(())
    }
}
