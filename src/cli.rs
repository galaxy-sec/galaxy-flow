use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use include_dir::{Dir, include_dir};
use orion_accessor::addr::GitRepository;
use orion_error::{ErrorConv, ToStructError};

use crate::GxLoader;
use crate::cmd::gx_cmd::{ConfCmd, DocArgs, GxCmd, InitCmd, SelfCmd, UpdateCmd};
use crate::cmd::gxl_cmd::GFlowCmd;
use crate::conf::{conf_init, conf_path, load_gxl_config};
use crate::const_val::gxl_const::{CMD_ARG, CONFIG_FILE};
use crate::err::{RunReason, RunResult};
use crate::execution::VarSpace;
use crate::galaxy::Galaxy;
use crate::help;
use crate::infra::configure_run_logging;
use crate::runner::GxlRunner;
use crate::self_update::{CheckRequest, ReleaseChannel, SelfUpdateService, UpdateRequest};
use crate::traits::Setter;
use crate::util::diagnose::ai_diagnose;
use crate::util::redirect::stop_redirect;

const ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/app/gx/init");

pub async fn run_from_env() -> RunResult<()> {
    let args = normalized_argv(std::env::args());
    let cmd = GxCmd::parse_from(args);
    dispatch(cmd).await
}

pub fn normalized_argv<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args: Vec<String> = args.into_iter().map(Into::into).collect();
    if args.is_empty() {
        args.push("gx".to_string());
    }

    let bin = Path::new(&args[0])
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("gx");

    match bin {
        "grun" => args.insert(1, "run".to_string()),
        "gadm" => args.insert(1, "adm".to_string()),
        _ => {}
    }

    args
}

pub async fn dispatch(cmd: GxCmd) -> RunResult<()> {
    if let GxCmd::Doc(args) = cmd.clone() {
        return do_doc_cmd(args);
    }

    println!("galaxy-flow : {}", env!("CARGO_PKG_VERSION"));

    let mut gx = GxLoader::new();
    match cmd {
        GxCmd::Run(cmd) => do_run_cmd(cmd).await?,
        GxCmd::Adm(cmd) => do_adm_cmd(cmd).await?,
        GxCmd::Init(prj_cmd) => do_prj_cmd(&mut gx, prj_cmd).await?,
        GxCmd::Update(prj_cmd) => do_update_cmd(&mut gx, prj_cmd).await?,
        GxCmd::Doc(_args) => unreachable!("doc is handled before command dispatch"),
        GxCmd::Conf(cmd) => do_conf_cmd(cmd).await?,
        GxCmd::Check => do_check_cmd()?,
        GxCmd::SelfUpdate(cmd) => do_self_cmd(cmd).await?,
    }
    Ok(())
}

async fn do_run_cmd(mut cmd: GFlowCmd) -> RunResult<()> {
    use std::process;

    let mut var_space = VarSpace::sys_init().err_conv()?;

    configure_run_logging(cmd.log.clone(), cmd.debug);
    load_gxl_config();

    let redirect = crate::model::task_report::task_rc_config::init_redirect_and_parent_task(
        cmd.flows.join(","),
        cmd.ai,
    )
    .await
    .err_conv()?;

    if cmd.conf.is_none() {
        cmd.conf = Some("./_gal/work.gxl".to_string());
    }
    var_space.global_mut().set(CMD_ARG, cmd.cmd_args.join(" "));

    if cmd.list_cmd().is_empty() {
        GxlRunner::info(cmd.conf.clone(), var_space).await?;
    } else {
        for cmd in cmd.list_cmd() {
            match GxlRunner::run(cmd.clone(), var_space.clone(), None).await {
                Err(e) => {
                    crate::err::report_gxl_error(e);
                    if cmd.ai
                        && let Err(e) = ai_diagnose(&var_space).await
                    {
                        crate::err::report_gxl_error(e);
                    }
                }
                Ok(_) => {
                    let _ = stop_redirect(redirect);
                    return Ok(());
                }
            }
        }
    }
    let _ = stop_redirect(redirect);
    process::exit(-1);
}

async fn do_adm_cmd(mut cmd: GFlowCmd) -> RunResult<()> {
    configure_run_logging(cmd.log.clone(), cmd.debug);
    let mut var_space = VarSpace::sys_init().err_conv()?;
    var_space.global_mut().set(CMD_ARG, cmd.cmd_args.join(" "));

    if cmd.conf.is_none() {
        cmd.conf = Some("./_gal/adm.gxl".to_string());
    }
    if cmd.list_cmd().is_empty() {
        GxlRunner::info(cmd.conf.clone(), var_space).await?;
    } else {
        for cmd in cmd.list_cmd() {
            match GxlRunner::run(cmd.clone(), var_space.clone(), None).await {
                Err(e) => {
                    crate::err::report_gxl_error(e);
                    if cmd.ai
                        && let Err(e) = ai_diagnose(&var_space).await
                    {
                        crate::err::report_gxl_error(e);
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
                println!("init {CONFIG_FILE}  success!");
            } else {
                println!("{CONFIG_FILE} exists!");
            }
        }
    }
    Ok(())
}

fn do_doc_cmd(args: DocArgs) -> RunResult<()> {
    help::print(args.topic.as_deref(), args.markdown)
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
            let channel = parse_channel(args.channel.as_str())?;
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
            let channel = parse_channel(args.channel.as_str())?;
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
    }
    Ok(())
}

fn parse_channel(input: &str) -> RunResult<ReleaseChannel> {
    ReleaseChannel::parse(input).ok_or_else(|| {
        RunReason::Args("bad channel".into())
            .to_err()
            .with_detail(format!("channel={input}, expected=stable|alpha|beta"))
    })
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
    fs::create_dir_all(parent_path.join(dir.path()))?;

    for file in dir.files() {
        let file_path = parent_path.join(file.path());
        fs::write(&file_path, file.contents())?;
    }

    for sub_dir in dir.dirs() {
        write_dir_to_disk(sub_dir, parent_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::normalized_argv;

    #[test]
    fn normalize_grun_to_run() {
        let args = normalized_argv(["grun", "conf"]);
        assert_eq!(args, vec!["grun", "run", "conf"]);
    }

    #[test]
    fn normalize_gadm_to_adm() {
        let args = normalized_argv(["gadm", "conf"]);
        assert_eq!(args, vec!["gadm", "adm", "conf"]);
    }
}
