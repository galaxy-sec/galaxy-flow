use std::collections::VecDeque;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::evaluator::{EnvExpress, VarParser};
use crate::expect::LogicScope;
use crate::expect::ShellOption;
use crate::var::VarDict;
use crate::{ExecReason, ExecResult};

use colored::*;
#[allow(clippy::result_large_err)]
pub fn os_sh(
    scope: LogicScope,
    target: &str,
    cmd: &str,
    opt: &ShellOption,
    exp: &EnvExpress,
    env: &VarDict,
) -> ExecResult<(Vec<u8>, Vec<u8>)> {
    let sec_cmd = exp.sec_eval(cmd)?;

    // 记录命令执行日志
    if !opt.secrecy {
        let lev = opt.log_lev.unwrap_or(log::Level::Debug);
        log!(target: target, lev, "cmd : {sec_cmd}");
        if !opt.quiet(scope) {
            show_cmd(&sec_cmd);
        }
    }

    let exe_cmd = exp.eval(cmd)?;
    let mut run_env = env.clone();
    run_env.merge_dict(VarDict::from(std::env::vars()));

    // 使用标准库的Command进行实时输出
    let mut child = std::process::Command::new("sh")
        .arg("-c")
        .arg(&exe_cmd)
        .envs(run_env.export_str_map())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ExecReason::OsCmd(sec_cmd.clone(), 254, e.to_string()))?;

    // 获取stdout和stderr的句柄
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ExecReason::OsCmd(sec_cmd.clone(), 254, "Failed to get stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| ExecReason::OsCmd(sec_cmd.clone(), 254, "Failed to get stderr".to_string()))?;

    // 用于收集输出的缓冲区
    let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
    let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

    let stdout_buffer_clone = Arc::clone(&stdout_buffer);
    let stderr_buffer_clone = Arc::clone(&stderr_buffer);

    let log_level = opt.log_lev.unwrap_or(log::Level::Debug);
    let target_str = target.to_string();
    let target_str_clone = target_str.clone();
    let quiet_mode = opt.quiet(scope);

    // 处理stdout的线程
    let stdout_handle = thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // 实时输出到控制台
                    if !quiet_mode {
                        println!("{}", line);
                        log!(target: &target_str, log_level, "out: {}", line);
                    }

                    // 保存到缓冲区
                    if let Ok(mut buffer) = stdout_buffer_clone.lock() {
                        buffer.extend_from_slice(line.as_bytes());
                        buffer.push(b'\n');
                    }
                }
                Err(e) => {
                    log::error!("Error reading stdout: {}", e);
                }
            }
        }
    });

    // 处理stderr的线程
    let stderr_handle = thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // 实时输出到控制台（stderr用黄色显示）
                    if !quiet_mode {
                        println!("{}", line.yellow());
                        log!(target: &target_str_clone, log_level, "err: {}", line);
                    }

                    // 保存到缓冲区
                    if let Ok(mut buffer) = stderr_buffer_clone.lock() {
                        buffer.extend_from_slice(line.as_bytes());
                        buffer.push(b'\n');
                    }
                }
                Err(e) => {
                    log::error!("Error reading stderr: {}", e);
                }
            }
        }
    });

    // 等待进程结束
    let exit_status = child
        .wait()
        .map_err(|e| ExecReason::OsCmd(sec_cmd.clone(), 254, e.to_string()))?;

    // 等待输出线程完成
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    // 获取收集的输出
    let stdout_data = stdout_buffer
        .lock()
        .map_err(|e| ExecReason::OsCmd(sec_cmd.clone(), 254, format!("Failed to lock stdout buffer: {}", e)))?
        .clone();
    let stderr_data = stderr_buffer
        .lock()
        .map_err(|e| ExecReason::OsCmd(sec_cmd.clone(), 254, format!("Failed to lock stderr buffer: {}", e)))?
        .clone();

    // 处理退出状态
    let fail_msg = opt.err.clone().unwrap_or(sec_cmd.clone());
    let fail_msg = exp.eval(fail_msg.as_str())?;

    if let Some(code) = exit_status.code() {
        let mut is_ok = code == 0;

        // 检查是否在期望的退出码列表中
        if !is_ok {
            for allow in &opt.expect {
                if code == *allow {
                    is_ok = true;
                    break;
                }
            }
        }

        // 显示成功消息
        if is_ok {
            if let Some(ref suc_msg) = opt.suc {
                println!("{suc_msg}");
            }
            Ok((stdout_data, stderr_data))
        } else {
            let err_msg = String::from_utf8(stderr_data.clone())
                .unwrap_or_else(|_| "Failed to decode stderr".to_string());
            Err(ExecReason::OsCmd(fail_msg, code, err_msg).into())
        }
    } else {
        Err(ExecReason::OsCmd(fail_msg, 252, "no exit code".to_string()).into())
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
                println!(".../{name} {simple_cmd}",);
                return;
            }
        }
        println!("{sec_cmd}",);
    }
}

#[cfg(test)]
mod tests {
    use orion_variate::vars::ValueType;

    use crate::var::VarDict;

    use super::*;
    //use duct_sh::* ;

    #[test]
    fn duct_test() {
        cmd!("echo", "hi").run().expect("Failed to run echo command");
        cmd!("/bin/sh", "./src/util/echo.sh").run().expect("Failed to run echo.sh");
        duct_sh::sh("echo hi2").run().expect("Failed to run sh echo");
        duct_sh::sh("cd ./src ; ls ").run().expect("Failed to run cd and ls");
    }
    #[test]
    fn rg_sh_test() {
        let mut dict = VarDict::global_new();
        dict.sec_set("SEC_KEY", ValueType::from("galaxy".to_string()));
        let exp = EnvExpress::from_env_mix(dict.clone());
        let opt = ShellOption {
            quiet: true,
            ..Default::default()
        };
        let cmd = "echo ${SEC_KEY}".to_string();
        let (stdout, _stderr) = os_sh(LogicScope::Outer, "gx.sh", &cmd, &opt, &exp, &dict).expect("Failed to execute os_sh");
        assert_eq!(stdout, b"galaxy\n");
    }

    #[test]
    fn test_realtime_output() {
        let dict = VarDict::global_new();
        let exp = EnvExpress::from_env_mix(dict.clone());
        let opt = ShellOption {
            quiet: false, // 启用输出以测试实时功能
            ..Default::default()
        };

        // 测试一个会产生多行输出的命令
        let cmd = "for i in 1 2 3; do echo \"Line $i\"; sleep 0.1; done".to_string();
        let (stdout, _stderr) =
            os_sh(LogicScope::Outer, "gx.sh.realtime", &cmd, &opt, &exp, &dict).expect("Failed to execute os_sh");

        let output_str = String::from_utf8(stdout).expect("Failed to decode stdout");
        assert!(output_str.contains("Line 1"));
        assert!(output_str.contains("Line 2"));
        assert!(output_str.contains("Line 3"));
    }

    #[test]
    fn test_realtime_error_handling() {
        let dict = VarDict::global_new();
        let exp = EnvExpress::from_env_mix(dict.clone());
        let opt = ShellOption {
            quiet: true,
            expect: vec![0], // 只期望成功退出
            ..Default::default()
        };

        // 测试一个会失败的命令
        let cmd = "exit 1".to_string();
        let result = os_sh(LogicScope::Outer, "gx.sh.realtime", &cmd, &opt, &exp, &dict);

        assert!(result.is_err());
        if let Err(e) = result {
            // 验证错误信息包含正确的退出码
            let error_str = format!("{:?}", e);
            assert!(error_str.contains("1")); // 退出码1
        }
    }

    #[test]
    fn test_realtime_with_stderr() {
        let dict = VarDict::global_new();
        let exp = EnvExpress::from_env_mix(dict.clone());
        let opt = ShellOption {
            quiet: false,
            ..Default::default()
        };

        // 测试同时产生stdout和stderr的命令
        let cmd = "echo 'stdout message'; echo 'stderr message' >&2".to_string();
        let (stdout, stderr) =
            os_sh(LogicScope::Outer, "gx.sh.realtime", &cmd, &opt, &exp, &dict).expect("Failed to execute os_sh");

        let stdout_str = String::from_utf8(stdout).expect("Failed to decode stdout");
        let stderr_str = String::from_utf8(stderr).expect("Failed to decode stderr");

        assert!(stdout_str.contains("stdout message"));
        assert!(stderr_str.contains("stderr message"));
    }
}
