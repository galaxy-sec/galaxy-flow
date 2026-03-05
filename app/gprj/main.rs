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
use args::SelfAutoCmd;
use args::SelfCmd;
use args::UpdateCmd;
use clap::Parser;
use galaxy_flow::GxLoader;
use galaxy_flow::cmd::gxl_cmd::GFlowCmd;
use galaxy_flow::conf::conf_init;
use galaxy_flow::conf::conf_path;
use galaxy_flow::const_val::gxl_const::CMD_ARG;
use galaxy_flow::const_val::gxl_const::CONFIG_FILE;
use galaxy_flow::err::*;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::galaxy::Galaxy;
use galaxy_flow::infra::configure_run_logging;
use galaxy_flow::runner::GxlRunner;
use galaxy_flow::self_update::{
    AutoMode, AutoSetRequest, CheckRequest, ReleaseChannel, SelfUpdateService, UpdateRequest,
};
use galaxy_flow::traits::Setter;
use galaxy_flow::util::diagnose::ai_diagnose;
use include_dir::{Dir, include_dir};
use orion_accessor::addr::GitRepository;
use orion_error::ErrorConv;
use orion_error::ToStructError;

const ASSETS_DIR: Dir = include_dir!("app/gprj/init");
#[tokio::main]
async fn main() {
    use std::process;
    let cmd = GxAdmCmd::parse();
    match GxAdm::run(cmd.clone()).await {
        Err(e) => {
            report_gxl_error(e);
        }
        Ok(_) => {
            return;
        }
    }
    process::exit(-1);
}

pub struct GxAdm {}
impl GxAdm {
    pub async fn run(cmd: GxAdmCmd) -> RunResult<()> {
        println!("galaxy-flow : {}", env!("CARGO_PKG_VERSION"));
        debug!("galaxy flow running .....");
        let mut gx = GxLoader::new();
        match cmd {
            GxAdmCmd::Init(prj_cmd) => {
                Self::do_prj_cmd(&mut gx, prj_cmd).await?;
            }
            GxAdmCmd::Update(prj_cmd) => {
                Self::do_update_cmd(&mut gx, prj_cmd).await?;
            }
            GxAdmCmd::Adm(cmd) => {
                Self::do_adm_cmd(cmd).await?;
            }
            GxAdmCmd::Conf(cmd) => {
                Self::do_conf_cmd(cmd).await?;
            }
            GxAdmCmd::Check => {
                Self::do_check_cmd()?;
            }
            GxAdmCmd::SelfUpdate(cmd) => {
                Self::do_self_cmd(cmd).await?;
            } //GxAdmCmd::Vault(cmd) => vault_main(cmd).owe_data()?,
              //GxAdmCmd::Sys(_cmd) => {
              //    let info = HardwareKit::machine_info().owe_sys()?;
              //    println!("sys info:\n{}", info);
              //}
        }
        Ok(())
    }

    async fn do_adm_cmd(mut cmd: GFlowCmd) -> RunResult<()> {
        configure_run_logging(cmd.log.clone(), cmd.debug);
        let mut var_space = VarSpace::sys_init().err_conv()?;
        var_space.global_mut().set(CMD_ARG, cmd.cmd_args.join(" "));

        debug!("galaxy flow running ....");
        if cmd.conf.is_none() {
            let main_conf = "./_gal/adm.gxl";
            cmd.conf = Some(main_conf.to_string());
        }
        if cmd.list_cmd().is_empty() {
            GxlRunner::info(cmd.conf.clone(), var_space).await?;
        } else {
            for cmd in cmd.list_cmd() {
                match GxlRunner::run(cmd.clone(), var_space.clone(), None).await {
                    Err(e) => {
                        report_gxl_error(e);
                        if cmd.ai
                            && let Err(e) = ai_diagnose(&var_space).await
                        {
                            report_gxl_error(e);
                        }
                    }

                    Ok(_) => {
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    async fn do_conf_cmd(cmd: ConfCmd) -> RunResult<()> {
        match cmd {
            ConfCmd::Init(_init_args) => {
                if conf_path().is_none() {
                    conf_init()?;
                    println!("init {CONFIG_FILE}  success!",);
                } else {
                    println!("{CONFIG_FILE} exists!",);
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
            println!("Architecture: {arch}");
        }
        println!("evn path:{}", env!("PATH"));
        Ok(())
    }

    async fn do_prj_cmd(load: &mut GxLoader, cmd: InitCmd) -> RunResult<()> {
        match cmd {
            InitCmd::Env => Galaxy::env_init()?,
            InitCmd::PrjWithLocal => {
                init_local(None)?;
            }
            InitCmd::Prj(args) => {
                configure_run_logging(args.log.clone(), args.debug);

                //let repo = ModRepo::new(args.repo.as_str(), args.channel.as_str()).owe_res()?;
                let addr = GitRepository::from(args.repo.as_str());
                let addr = if let Some(tag) = args.tag() {
                    addr.with_tag(tag)
                } else if let Some(branch) = args.branch() {
                    addr.with_branch(branch)
                } else {
                    addr
                };
                load.init(addr, args.tpl.as_str()).await?;
            }
        }
        Ok(())
    }

    async fn do_update_cmd(load: &mut GxLoader, prj_cmd: UpdateCmd) -> RunResult<()> {
        match prj_cmd {
            UpdateCmd::Mod(args) => {
                configure_run_logging(args.log.clone(), args.debug);

                let vars = VarSpace::sys_init().err_conv()?;

                if std::path::Path::new(args.conf_work.as_str()).exists() {
                    load.parse_file(args.conf_work.as_str(), true, &vars)
                        .await?;
                }
                if std::path::Path::new(args.conf_adm.as_str()).exists() {
                    load.parse_file(args.conf_adm.as_str(), true, &vars).await?;
                }
            }
        }
        Ok(())
    }

    async fn do_self_cmd(cmd: SelfCmd) -> RunResult<()> {
        let svc = SelfUpdateService::new()?;
        match cmd {
            SelfCmd::Status => {
                let status = svc.status()?;
                println!("current_version={}", status.current_version);
                println!("install_dir={}", status.install_dir.display());
                println!("policy.enabled={}", status.policy.enabled);
                println!("policy.mode={:?}", status.policy.mode);
                println!("policy.channel={}", status.policy.channel.as_str());
                println!("policy.interval_hours={}", status.policy.interval_hours);
                if let Some(v) = status.state.last_remote_version {
                    println!("state.last_remote_version={v}");
                }
                if let Some(v) = status.state.last_result {
                    println!("state.last_result={v}");
                }
                if let Some(v) = status.state.last_error {
                    println!("state.last_error={v}");
                }
            }
            SelfCmd::Check(args) => {
                let channel = parse_channel(args.channel.as_deref())?;
                let req = CheckRequest { channel };
                let out = svc.check(req).await?;
                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "channel": out.channel.as_str(),
                            "current_version": out.current_version,
                            "remote_version": out.remote_version,
                            "has_update": out.has_update
                        }))
                        .map_err(|e| RunReason::Exec(e.to_string()).to_err())?
                    );
                } else {
                    println!("channel={}", out.channel.as_str());
                    println!("current={}", out.current_version);
                    println!("remote={}", out.remote_version);
                    println!("has_update={}", out.has_update);
                }
            }
            SelfCmd::Update(args) => {
                let channel = parse_channel(args.channel.as_deref())?;
                let req = UpdateRequest {
                    channel,
                    to_version: args.to_version.clone(),
                    yes: args.yes,
                    dry_run: args.dry_run,
                    force: args.force,
                };
                let out = svc.update(req).await?;
                println!("channel={}", out.channel.as_str());
                println!("from={}", out.from_version);
                println!("to={}", out.to_version);
                println!("updated={}", out.updated);
                if let Some(id) = out.backup_id {
                    println!("backup_id={id}");
                }
            }
            SelfCmd::Rollback(args) => {
                let out = svc.rollback(args.backup_id.as_deref())?;
                println!("rollback=true");
                if let Some(id) = out.backup_id {
                    println!("backup_id={id}");
                }
            }
            SelfCmd::Auto(cmd) => match cmd {
                SelfAutoCmd::Enable => {
                    let out = svc.set_auto(AutoSetRequest {
                        enabled: Some(true),
                        ..Default::default()
                    })?;
                    println!("auto.enabled={}", out.enabled);
                }
                SelfAutoCmd::Disable => {
                    let out = svc.set_auto(AutoSetRequest {
                        enabled: Some(false),
                        ..Default::default()
                    })?;
                    println!("auto.enabled={}", out.enabled);
                }
                SelfAutoCmd::Set(args) => {
                    let req = AutoSetRequest {
                        enabled: None,
                        mode: parse_mode(args.mode.as_deref())?,
                        interval_hours: args.interval,
                        channel: parse_channel(args.channel.as_deref())?,
                    };
                    let out = svc.set_auto(req)?;
                    println!("auto.enabled={}", out.enabled);
                    println!("auto.mode={:?}", out.mode);
                    println!("auto.channel={}", out.channel.as_str());
                    println!("auto.interval_hours={}", out.interval_hours);
                }
            },
        }
        Ok(())
    }
}

fn parse_channel(input: Option<&str>) -> RunResult<Option<ReleaseChannel>> {
    match input {
        None => Ok(None),
        Some(v) => ReleaseChannel::parse(v).map(Some).ok_or_else(|| {
            RunReason::Args("bad channel".into())
                .to_err()
                .with_detail(format!("channel={v}, expected=stable|alpha|beta"))
        }),
    }
}

fn parse_mode(input: Option<&str>) -> RunResult<Option<AutoMode>> {
    match input {
        None => Ok(None),
        Some(v) => match v.trim().to_ascii_lowercase().as_str() {
            "check" => Ok(Some(AutoMode::Check)),
            "apply" => Ok(Some(AutoMode::Apply)),
            _ => Err(RunReason::Args("bad auto mode".into())
                .to_err()
                .with_detail(format!("mode={v}, expected=check|apply"))),
        },
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
        let mut gflow_cmd = GFlowCmd::try_parse_from(["gxl", "-e", "default", "echo"]).unwrap();
        gflow_cmd.conf = Some("./_gal/adm.gxl".to_string());
        GxAdm::do_adm_cmd(gflow_cmd).await.assert();
    }
}
