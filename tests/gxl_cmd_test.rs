extern crate galaxy_flow;

use clap::Parser;
use galaxy_flow::cmd::gxl_cmd::GFlowCmd;

#[derive(Parser, Debug)]
struct GFlowCmdCli {
    #[command(flatten)]
    cmd: GFlowCmd,
}

#[test]
fn test_gxl_cmd_default() {
    // 测试 GxlCmd 的默认值
    let cmd = GFlowCmdCli::try_parse_from(["gxl"])
        .expect("Failed to parse default command")
        .cmd;

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
    let cmd = GFlowCmdCli::try_parse_from([
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
    .expect("Failed to parse command with args")
    .cmd;

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
    let cmd = GFlowCmdCli::try_parse_from(["gxl", "-e", "test", "--cmd-arg", "-custom", "flow1"])
        .expect("Failed to parse command with hyphen args")
        .cmd;

    assert_eq!(cmd.cmd_args, vec!["-custom".to_string()]);
    assert_eq!(cmd.flows, vec!["flow1".to_string()]);
}

#[test]
fn test_gxl_cmd_multiple_flows() {
    // 测试 GxlCmd 多个流程的情况
    let cmd = GFlowCmdCli::try_parse_from(["gxl", "-e", "prod", "build,test,deploy"])
        .expect("Failed to parse command with multiple flows")
        .cmd;

    assert!(cmd.cmd_args.is_empty());
    assert_eq!(cmd.flows, vec!["build,test,deploy".to_string()]);
}

#[test]
fn test_gxl_cmd_separate_flows() {
    // 测试 GxlCmd 分离的多个流程的情况
    let cmd = GFlowCmdCli::try_parse_from(["gxl", "-e", "staging", "build", "test", "deploy"])
        .expect("Failed to parse command with separate flows")
        .cmd;

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
