extern crate galaxy_flow;

use clap::Parser;
use galaxy_flow::cmd::GxlCmd;

#[test]
fn test_gxl_cmd_default() {
    // 测试 GxlCmd 的默认值
    let cmd = GxlCmd::try_parse_from(&["gxl"]).expect("Failed to parse default command");

    assert_eq!(cmd.env, "default");
    assert_eq!(cmd.debug, 0);
    assert_eq!(cmd.dryrun, false);
    assert_eq!(cmd.ai, false);
    assert_eq!(cmd.mod_update, false);
    assert!(cmd.cmd_args.is_empty());
    assert!(cmd.conf.is_none());
    assert!(cmd.log.is_none());
    assert_eq!(cmd.quiet, false);
    assert!(cmd.flows.is_empty());
    assert!(cmd.flows.is_empty());
}

#[test]
fn test_gxl_cmd_with_args() {
    // 测试 GxlCmd 带参数的情况
    let cmd = GxlCmd::try_parse_from(&[
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

    assert_eq!(cmd.env, "dev");
    assert_eq!(cmd.debug, 1);
    assert_eq!(cmd.conf, Some("./_gal/work.gxl".to_string()));
    assert_eq!(cmd.log, Some("cmd=debug".to_string()));
    assert_eq!(cmd.quiet, true);
    assert_eq!(cmd.dryrun, true);
    assert_eq!(cmd.ai, true);
    assert_eq!(cmd.mod_update, true);
    assert!(cmd.cmd_args.is_empty());
    assert_eq!(cmd.flows, vec!["flow1".to_string(), "flow2".to_string()]);
}

#[test]
fn test_gxl_cmd_with_hyphen_args() {
    // 测试 GxlCmd 带连字符参数的情况
    let cmd = GxlCmd::try_parse_from(&["gxl", "-e", "test", "--cmd-arg", "-custom", "flow1"])
        .expect("Failed to parse command with hyphen args");

    assert_eq!(cmd.env, "test");
    assert_eq!(cmd.cmd_args, vec!["-custom".to_string()]);
    assert_eq!(cmd.flows, vec!["flow1".to_string()]);
}

#[test]
fn test_gxl_cmd_multiple_flows() {
    // 测试 GxlCmd 多个流程的情况
    let cmd = GxlCmd::try_parse_from(&["gxl", "-e", "prod", "build,test,deploy"])
        .expect("Failed to parse command with multiple flows");

    assert_eq!(cmd.env, "prod");
    assert!(cmd.cmd_args.is_empty());
    assert_eq!(cmd.flows, vec!["build,test,deploy".to_string()]);
}

#[test]
fn test_gxl_cmd_separate_flows() {
    // 测试 GxlCmd 分离的多个流程的情况
    let cmd = GxlCmd::try_parse_from(&["gxl", "-e", "staging", "build", "test", "deploy"])
        .expect("Failed to parse command with separate flows");

    assert_eq!(cmd.env, "staging");
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
    // 测试 GxlCmd 参数验证

    // 测试缺少配置文件的情况
    let cmd = GxlCmd {
        env: "test".to_string(),
        flows: vec!["flow1".to_string()],
        debug: 0,
        conf: None,
        log: None,
        quiet: false,
        cmd_args: vec![],
        dryrun: false,
        ai: false,
        mod_update: false,
    };

    let result = cmd.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Configuration file is required");

    // 测试没有flow但指定了cmd_args的情况
    let cmd = GxlCmd {
        env: "test".to_string(),
        flows: vec![],
        debug: 0,
        conf: Some("./_gal/work.gxl".to_string()),
        log: None,
        quiet: false,
        cmd_args: vec!["-custom".to_string()],
        dryrun: false,
        ai: false,
        mod_update: false,
    };

    let result = cmd.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot specify cmd_args without flows");

    // 测试有效参数的情况
    let cmd = GxlCmd {
        env: "test".to_string(),
        flows: vec!["flow1".to_string()],
        debug: 0,
        conf: Some("./_gal/work.gxl".to_string()),
        log: None,
        quiet: false,
        cmd_args: vec!["-custom".to_string()],
        dryrun: false,
        ai: false,
        mod_update: false,
    };

    let result = cmd.validate();
    assert!(result.is_ok());
}
