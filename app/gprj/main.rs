mod args;
//mod vault;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::args::GxAdmCmd;
use crate::args::InitCmd;
use args::ConfCmd;
use clap::Parser;
use galaxy_flow::conf::conf_init;
use galaxy_flow::conf::conf_path;
use galaxy_flow::const_val::CONFIG_FILE;
use galaxy_flow::err::*;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::expect::ShellOption;
use galaxy_flow::infra::configure_run_logging;
use galaxy_flow::runner::{GxlCmd, GxlRunner};
use galaxy_flow::util::{GitTools, ModRepo};
use galaxy_flow::GxLoader;
use include_dir::{include_dir, Dir};
use orion_error::{ErrorConv, ErrorOwe};

const ASSETS_DIR: Dir = include_dir!("app/gprj/init");
#[tokio::main]
async fn main() {
    use std::process;
    match GxAdm::run().await {
        Err(e) => report_gxl_error(e),
        Ok(_) => {
            return;
        }
    }
    process::exit(-1);
}

pub struct GxAdm {}
impl GxAdm {
    pub async fn run() -> RunResult<()> {
        let cmd = GxAdmCmd::parse();
        debug!("galaxy flow running .....");
        let mut gx = GxLoader::new();
        match cmd {
            GxAdmCmd::Init(prj_cmd) => {
                Self::do_prj_cmd(&mut gx, prj_cmd)?;
            }
            GxAdmCmd::Adm(cmd) => {
                Self::do_adm_cmd(cmd).await?;
            }
            GxAdmCmd::Conf(cmd) => {
                Self::do_conf_cmd(cmd).await?;
            }
            GxAdmCmd::Check => {
                Self::do_check_cmd()?;
            } //GxAdmCmd::Vault(cmd) => vault_main(cmd).owe_data()?,
              //GxAdmCmd::Sys(_cmd) => {
              //    let info = HardwareKit::machine_info().owe_sys()?;
              //    println!("sys info:\n{}", info);
              //}
        }
        Ok(())
    }

    async fn do_adm_cmd(mut cmd: GxlCmd) -> RunResult<()> {
        configure_run_logging(cmd.log.clone(), cmd.debug);
        debug!("galaxy flow running .....");
        if cmd.conf.is_none() {
            cmd.conf = Some("./_gal/adm.gxl".to_string());
        }
        let var_space = VarSpace::sys_init().err_conv()?;
        GxlRunner::run(cmd, var_space).await?;
        Ok(())
    }

    async fn do_conf_cmd(cmd: ConfCmd) -> RunResult<()> {
        match cmd {
            ConfCmd::Init(_init_args) => {
                if conf_path().is_none() {
                    conf_init()?;
                    println!("init {}  success!", CONFIG_FILE);
                } else {
                    println!("{} exists!", CONFIG_FILE);
                }
            }
        }
        Ok(())
    }

    fn do_check_cmd() -> RunResult<()> {
        let info = os_info::get();
        println!("galaxy flow running env info");
        println!("OS : {info}");
        println!("Type: {}", info.os_type());
        println!("Version: {}", info.version());
        println!("Bitness: {}", info.bitness());
        if let Some(arch) = info.architecture() {
            println!("Architecture: {}", arch);
        }
        println!("evn path:{}", env!("PATH"));
        let gtools = GitTools::new(false).err_conv()?;
        gtools.check_run().err_conv()?;
        Ok(())
    }

    fn do_prj_cmd(load: &mut GxLoader, prj_cmd: InitCmd) -> RunResult<()> {
        match prj_cmd {
            InitCmd::Local => {
                init_local(None)?;
            }
            InitCmd::Remote(args) => {
                configure_run_logging(args.log.clone(), args.debug);
                let sh_opt = ShellOption {
                    quiet: args.cmd_print,
                    inner_print: args.cmd_print,
                    ..Default::default()
                };
                let repo = ModRepo::new(args.repo.as_str(), args.channel.as_str()).owe_res()?;
                load.init(repo, "./", true, args.tpl.as_str(), sh_opt)?;
            }
            InitCmd::Update(args) => {
                configure_run_logging(args.log.clone(), args.debug);
                let sh_opt = ShellOption {
                    quiet: args.cmd_print,
                    inner_print: args.cmd_print,
                    ..Default::default()
                };
                let vars = VarSpace::sys_init().err_conv()?;

                if std::path::Path::new(args.conf_work.as_str()).exists() {
                    load.parse_file(args.conf_work.as_str(), true, sh_opt.clone(), &vars)?;
                }
                if std::path::Path::new(args.conf_adm.as_str()).exists() {
                    load.parse_file(args.conf_adm.as_str(), true, sh_opt, &vars)?;
                }
            }
        }
        Ok(())
    }
}

fn init_local(path: Option<PathBuf>) -> RunResult<()> {
    let src_path = match path {
        Some(path) => path,
        None => std::env::current_dir().expect("Failed to get current directory"),
    };
    write_dir_to_disk(&ASSETS_DIR, &src_path).expect("Failed to write directory to disk");
    Ok(())
}

fn write_dir_to_disk(dir: &Dir, parent_path: &Path) -> std::io::Result<()> {
    // 创建目录
    fs::create_dir_all(parent_path.join(dir.path()))?;

    // 写入文件
    for file in dir.files() {
        let file_path = parent_path.join(file.path());
        fs::write(&file_path, file.contents())?;
    }

    // 递归写入子目录
    for sub_dir in dir.dirs() {
        write_dir_to_disk(sub_dir, parent_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use galaxy_flow::util::path::WorkDir;
    use orion_error::TestAssert;

    use super::*;

    // 调用 init_local   初始化本地项目
    //  do_adm_cmd 函数进行测试
    #[tokio::test]
    async fn test_init_local() {
        let init_local_path = PathBuf::from("./tests/temp/init");
        if init_local_path.exists() {
            fs::remove_dir_all(&init_local_path).unwrap();
        }
        fs::create_dir_all(&init_local_path).unwrap();
        let result = init_local(Some(init_local_path.clone()));
        assert!(result.is_ok());
        let _cur = WorkDir::change(init_local_path).assert();
        GxAdm::do_adm_cmd(GxlCmd {
            conf: Some("./_gal/adm.gxl".to_string()),
            log: None,
            debug: 0,
            env: "default".into(),
            flow: vec!["echo".into()],
            quiet: Some(true),
            cmd_arg: String::new(),
            dryrun: false,
        })
        .await
        .assert();
    }
}
