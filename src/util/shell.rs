use duct_sh;
use std::collections::VecDeque;

use crate::evaluator::{EnvExpress, Parser};
use crate::expect::LogicScope;
use crate::expect::ShellOption;
use crate::{ExecReason, ExecResult};

use colored::*;
#[allow(clippy::result_large_err)]
pub fn rg_sh(
    scope: LogicScope,
    target: &str,
    cmd: &str,
    opt: &ShellOption,
    exp: &EnvExpress,
) -> ExecResult<Vec<u8>> {
    let sec_cmd = exp.sec_eval(cmd)?;
    //let ee = EnvExpress::from_env();
    if !opt.secrecy {
        let lev = opt.log_lev.unwrap_or(log::Level::Debug);
        log!(target: target, lev, "cmd : {}", sec_cmd);
        if opt.cmd_print(scope) {
            show_cmd(&sec_cmd);
        }
    }
    let exe_cmd = exp.eval(cmd)?;
    //use unchecked() not return error
    let output = duct_sh::sh_dangerous(exe_cmd)
        .unchecked()
        .stdout_capture()
        .stderr_capture()
        .run();
    let fail_msg = opt.err.clone().unwrap_or(sec_cmd.clone());
    let fail_msg = exp.eval(fail_msg.as_str())?;
    match output {
        Err(e) => Err(ExecReason::OsCmd(fail_msg, 254, e.to_string()).into()),
        Ok(out) => {
            let mut is_ok = false;
            if let Some(code) = out.status.code() {
                let err_desp = "err msg from utf8 failed";
                let out_msg = String::from_utf8(out.stdout.clone())
                    .map_err(|_| ExecReason::OsCmd(sec_cmd.clone(), 253, err_desp.to_string()))?;
                let err_msg = String::from_utf8(out.stderr.clone())
                    .map_err(|_| ExecReason::OsCmd(sec_cmd.clone(), code, err_desp.to_string()))?;
                if code == 0 {
                    if let Some(ref suc_msg) = opt.suc {
                        println!("{}", suc_msg);
                    }
                    is_ok = true;
                } else {
                    for allow in &opt.expect {
                        if code == *allow {
                            is_ok = true;
                        }
                    }
                }

                let log_level = opt.log_lev.unwrap_or(log::Level::Debug);
                if opt.cmd_print(scope) {
                    if !out_msg.is_empty() {
                        println!("{}", out_msg);
                        log!(target: target, log_level, "out:\n{}", out_msg);
                    }
                    if !err_msg.is_empty() {
                        if is_ok {
                            println!("{}", err_msg.yellow());
                            log!(target: target, log_level, "out:\n{}", err_msg);
                        } else {
                            println!("{}", err_msg.clone().red());
                            log!(target: target, log_level, "err:\n{}", err_msg);
                        }
                    }
                }
                return if is_ok {
                    Ok(out.stdout)
                } else {
                    Err(ExecReason::OsCmd(fail_msg, code, err_msg).into())
                };
            }
            Err(ExecReason::OsCmd(fail_msg, 252, "no exit code".to_string()).into())
        }
    }
}

fn show_cmd(sec_cmd: &String) {
    let mut cmd_elements = VecDeque::from(sec_cmd.split(' ').collect::<Vec<&str>>());
    let first_element = cmd_elements.pop_front();
    if let Some(first) = first_element {
        let cmd_first = std::path::Path::new(first);
        if cmd_first.exists() {
            let cmd_name = cmd_first.file_name().and_then(|x| x.to_str());
            if let Some(name) = cmd_name {
                let simple_cmd = Vec::from(cmd_elements).join(" ");
                println!(".../{} {}", name, simple_cmd);
                return;
            }
        }
        println!("{}", sec_cmd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::var::VarsDict;
    //use duct_sh::* ;

    #[test]
    fn duct_test() {
        cmd!("echo", "hi").run().unwrap();
        cmd!("/bin/sh", "./src/util/echo.sh").run().unwrap();
        duct_sh::sh("echo hi2").run().unwrap();
        duct_sh::sh("cd ./src ; ls ").run().unwrap();
    }
    #[test]
    fn rg_sh_test() {
        let mut dict = VarsDict::new();
        dict.sec_set("SEC_KEY", "galaxy");
        let exp = EnvExpress::from_env_mix(dict);
        let opt = ShellOption {
            outer_print: true,
            ..Default::default()
        };
        let cmd = "echo ${SEC_KEY}".to_string();
        let out = rg_sh(LogicScope::Outer, "gx.sh", &cmd, &opt, &exp).unwrap();
        assert_eq!(out, b"galaxy\n");
    }
}
