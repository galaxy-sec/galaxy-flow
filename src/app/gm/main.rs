mod args;
//mod vault;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use std::fs;
use std::path::Path;

use crate::args::{GxAdmCmd, PrjCmd};
use clap::Parser;
use galaxy_flow::err::*;
use galaxy_flow::expect::ShellOption;
use galaxy_flow::infra::configure_flow_logging;
use galaxy_flow::runner::{GxlCmd, GxlRunner};
use galaxy_flow::util::{GitTools, ModRepo};
use galaxy_flow::GxLoader;
use include_dir::{include_dir, Dir};
use orion_error::{ErrorConv, ErrorOwe};

const ASSETS_DIR: Dir = include_dir!("src/app/gm/init");
fn main() {
    use std::process;
    match GxAdm::run() {
        Err(e) => report_rg_error(e),
        Ok(_) => {
            return;
        }
    }
    process::exit(-1);
}

pub struct GxAdm {}
impl GxAdm {
    pub fn run() -> RunResult<()> {
        let cmd = GxAdmCmd::parse();
        debug!("galaxy flow running .....");
        let mut gx = GxLoader::new();
        match cmd {
            GxAdmCmd::Prj(prj_cmd) => {
                Self::do_prj_cmd(&mut gx, prj_cmd)?;
            }
            GxAdmCmd::Adm(cmd) => {
                Self::do_adm_cmd(cmd)?;
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

    fn do_adm_cmd(mut cmd: GxlCmd) -> RunResult<()> {
        configure_flow_logging(cmd.log.clone(), cmd.debug);
        debug!("galaxy flow running .....");
        if cmd.conf.is_none() {
            cmd.conf = Some("./_gal/adm.gxl".to_string());
        }
        GxlRunner::run(cmd)?;
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

    fn do_prj_cmd(load: &mut GxLoader, prj_cmd: PrjCmd) -> RunResult<()> {
        match prj_cmd {
            PrjCmd::Init => {
                let current_dir = std::env::current_dir().expect("Failed to get current directory");
                write_dir_to_disk(&ASSETS_DIR, &current_dir)
                    .expect("Failed to write directory to disk");
            }
            PrjCmd::RemoteInit(args) => {
                configure_flow_logging(args.log.clone(), args.debug);
                let sh_opt = ShellOption {
                    outer_print: args.cmd_print,
                    inner_print: args.cmd_print,
                    ..Default::default()
                };
                let repo = ModRepo::new(args.repo.as_str(), args.channel.as_str()).owe_res()?;
                load.init(repo, "./", true, args.tpl.as_str(), sh_opt)?;
            }
            PrjCmd::Update(args) => {
                configure_flow_logging(args.log.clone(), args.debug);
                let sh_opt = ShellOption {
                    outer_print: args.cmd_print,
                    inner_print: args.cmd_print,
                    ..Default::default()
                };

                if std::path::Path::new(args.conf_work.as_str()).exists() {
                    load.parse_file(args.conf_work.as_str(), true, sh_opt.clone())?;
                }
                if std::path::Path::new(args.conf_adm.as_str()).exists() {
                    load.parse_file(args.conf_adm.as_str(), true, sh_opt)?;
                }
            }
        }
        Ok(())
    }
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
