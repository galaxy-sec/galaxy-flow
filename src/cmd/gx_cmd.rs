use clap::{ArgAction, Args, Parser, Subcommand};
use derive_getters::Getters;

use crate::cmd::gxl_cmd::GFlowCmd;

#[derive(Debug, Parser, Clone)]
#[command(name = "gx")]
#[command(version, about = "Galaxy Flow unified CLI")]
#[command(disable_help_subcommand = true)]
pub enum GxCmd {
    Run(GFlowCmd),
    Adm(GFlowCmd),
    #[command(subcommand)]
    Init(InitCmd),
    #[command(subcommand)]
    Update(UpdateCmd),
    #[command(name = "doc", alias = "docs")]
    Doc(DocArgs),
    #[command(subcommand)]
    Conf(ConfCmd),
    Check,
    #[command(name = "self", subcommand)]
    SelfUpdate(SelfCmd),
}

#[derive(Debug, Subcommand, Clone)]
pub enum InitCmd {
    /// init galaxy env
    Env,
    /// init project with local mod
    PrjWithLocal,
    /// init project with remote mod
    Prj(InitArgs),
}

#[derive(Debug, Subcommand, Clone)]
pub enum UpdateCmd {
    Mod(PrjArgs),
}

#[derive(Debug, Subcommand, Clone)]
pub enum ConfCmd {
    Init(ConfInitArgs),
}

#[derive(Debug, Args, Clone, Getters)]
pub struct DocArgs {
    #[arg(long, action = ArgAction::SetTrue, default_value = "false")]
    pub markdown: bool,
    pub topic: Option<String>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum SelfCmd {
    Status,
    Check(SelfCheckArgs),
    Update(SelfUpdateArgs),
    Rollback(SelfRollbackArgs),
}

#[derive(Debug, Args, Clone, Getters)]
pub struct SelfCheckArgs {
    #[arg(long)]
    pub channel: String,
    #[arg(long, action = ArgAction::SetTrue, default_value = "false")]
    pub json: bool,
}

#[derive(Debug, Args, Clone, Getters)]
pub struct SelfUpdateArgs {
    #[arg(long)]
    pub channel: String,
    #[arg(long = "to")]
    pub to_version: Option<String>,
    #[arg(long, action = ArgAction::SetTrue, default_value = "false")]
    pub yes: bool,
    #[arg(long = "dry-run", action = ArgAction::SetTrue, default_value = "false")]
    pub dry_run: bool,
    #[arg(long, action = ArgAction::SetTrue, default_value = "false")]
    pub force: bool,
}

#[derive(Debug, Args, Clone, Getters)]
pub struct SelfRollbackArgs {
    #[arg(long = "id")]
    pub backup_id: Option<String>,
}

#[derive(Debug, Args, Getters, Clone)]
pub struct InitArgs {
    /// chose init tpl from rg-tpl repo. eg: --tpl open_pages
    #[arg(short, long, default_value = "simple")]
    pub(crate) tpl: String,
    /// branch or tag for rg-tpl repo
    #[arg(short, long)]
    pub(crate) branch: Option<String>,

    #[arg(short, long)]
    pub(crate) tag: Option<String>,
    /// debug level ; eg: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub(crate) debug: usize,

    #[arg(long = "repo", default_value = "https://gal-tpl.git")]
    pub repo: String,
    #[arg(long = "log")]
    pub log: Option<String>,
    #[arg(short= 'p', long = "cmd_print" ,action = ArgAction::SetTrue, default_value = "false")]
    pub cmd_print: bool,
}

#[derive(Debug, Args, Getters, Clone)]
pub struct ConfInitArgs {
    #[arg(short = 'r', long = "remote", default_value = "false")]
    pub remote: bool,
}

#[derive(Debug, Args, Clone)]
pub struct PrjArgs {
    /// debug level ; eg: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub(crate) debug: usize,
    /// conf file ; eg: -f ./_gal/prj.gxl
    #[arg(long, default_value = "./_gal/work.gxl")]
    pub(crate) conf_work: String,
    #[arg(long, default_value = "./_gal/adm.gxl")]
    pub(crate) conf_adm: String,
    #[arg(long = "log")]
    pub log: Option<String>,
    #[arg(short= 'q', long = "quiet" ,action = ArgAction::SetFalse , default_value = "true")]
    pub cmd_print: bool,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::GxCmd;

    #[test]
    fn parse_doc_topic() {
        let cmd = GxCmd::try_parse_from(["gx", "doc", "gx.cmd"]).expect("doc command should parse");

        match cmd {
            GxCmd::Doc(args) => {
                assert_eq!(args.topic.as_deref(), Some("gx.cmd"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parse_doc_markdown() {
        let cmd = GxCmd::try_parse_from(["gx", "doc", "--markdown", "gx.cmd"])
            .expect("doc markdown command should parse");

        match cmd {
            GxCmd::Doc(args) => {
                assert!(args.markdown);
                assert_eq!(args.topic.as_deref(), Some("gx.cmd"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
