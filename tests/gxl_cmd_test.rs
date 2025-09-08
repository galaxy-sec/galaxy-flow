extern crate galaxy_flow;

use clap::Parser;
use galaxy_flow::cmd::GxlCmd;

#[test]
fn test_gxl_cmd_default() {
    // 测试 GxlCmd 的默认值
    let cmd = GxlCmd::try_parse_from(["gxl"]).expect("Failed to parse default command");

    assert_eq!(cmd.debug, 0);
    assert!(!cmd.dryrun);
    assert!(!cmd.ai);
    assert!(!cmd.mod_update);
    assert!(cmd.cmd_args.is_empty());
    assert!(cmd.conf.is_none());
    assert!(cmd.log.is_none());
    assert!(!cmd.quiet);
    assert!(cmd.flows.is_empty());
    assert!(cmd.flows.is_empty());
}

#[test]
fn test_gxl_cmd_with_args() {
    // 测试 GxlCmd 带参数的情况
    let cmd = GxlCmd::try_parse_from([
        "gxl",
        "-e",
        "dev",
        "-d",
        "1",
        "-c",
        "./_gal/work.gxl",
        "--log",
        "cmd=debug",
        "-q",
        "--dryrun",
        "--ai",
        "--mod_up",
        "flow1",
        "flow2",
    ])
    .expect("Failed to parse command with args");

    assert_eq!(cmd.debug, 1);
    assert_eq!(cmd.conf, Some("./_gal/work.gxl".to_string()));
    assert_eq!(cmd.log, Some("cmd=debug".to_string()));
    assert!(cmd.quiet);
    assert!(cmd.dryrun);
    assert!(cmd.ai);
    assert!(cmd.mod_update);
    assert!(cmd.cmd_args.is_empty());
    assert_eq!(cmd.flows, vec!["flow1".to_string(), "flow2".to_string()]);
}

#[test]
fn test_gxl_cmd_with_hyphen_args() {
    // 测试 GxlCmd 带连字符参数的情况
    let cmd = GxlCmd::try_parse_from(["gxl", "-e", "test", "--cmd-arg", "-custom", "flow1"])
        .expect("Failed to parse command with hyphen args");

    assert_eq!(cmd.cmd_args, vec!["-custom".to_string()]);
    assert_eq!(cmd.flows, vec!["flow1".to_string()]);
}

#[test]
fn test_gxl_cmd_multiple_flows() {
    // 测试 GxlCmd 多个流程的情况
    let cmd = GxlCmd::try_parse_from(["gxl", "-e", "prod", "build,test,deploy"])
        .expect("Failed to parse command with multiple flows");

    assert!(cmd.cmd_args.is_empty());
    assert_eq!(cmd.flows, vec!["build,test,deploy".to_string()]);
}

#[test]
fn test_gxl_cmd_separate_flows() {
    // 测试 GxlCmd 分离的多个流程的情况
    let cmd = GxlCmd::try_parse_from(["gxl", "-e", "staging", "build", "test", "deploy"])
        .expect("Failed to parse command with separate flows");

    assert!(cmd.cmd_args.is_empty());
    assert_eq!(
        cmd.flows,
        vec![
            "build".to_string(),
            "test".to_string(),
            "deploy".to_string()
        ]
    );
}

#[test]
fn test_gxl_cmd_validation() {
    let cmd = GxlCmd::default();
    let result = cmd.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Configuration file is required");

    let mut cmd = GxlCmd::default();
    cmd.conf = Some("config.gxl".to_string());
    cmd.cmd_args = vec!["-x".to_string()];
    let result = cmd.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot specify cmd_args without flows");

    let mut cmd = GxlCmd::default();
    cmd.conf = Some("config.gxl".to_string());
    cmd.flows = vec!["flow1".to_string()];
    let result = cmd.validate();
    assert!(result.is_ok());
}

#[test]
fn test_get_env_list() {
    let mut cmd = GxlCmd::default();

    // 测试空字符串
    cmd.set_env("".to_string());
    assert_eq!(cmd.get_env_list(), Vec::<String>::new());

    // 测试单个环境
    cmd.set_env("dev".to_string());
    assert_eq!(cmd.get_env_list(), vec!["dev".to_string()]);

    // 测试多个环境，带逗号
    cmd.set_env("dev,test,prod".to_string());
    assert_eq!(
        cmd.get_env_list(),
        vec!["dev".to_string(), "test".to_string(), "prod".to_string()]
    );

    // 测试带空格的环境
    cmd.set_env("dev, test, prod".to_string());
    assert_eq!(
        cmd.get_env_list(),
        vec!["dev".to_string(), "test".to_string(), "prod".to_string()]
    );

    // 测试空环境值
    cmd.set_env("dev,,prod".to_string());
    assert_eq!(
        cmd.get_env_list(),
        vec!["dev".to_string(), "prod".to_string()]
    );

    // 测试只有空格的环境
    cmd.set_env("dev,  , prod".to_string());
    assert_eq!(
        cmd.get_env_list(),
        vec!["dev".to_string(), "prod".to_string()]
    );
}
