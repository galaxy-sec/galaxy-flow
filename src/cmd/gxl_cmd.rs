use clap::{ArgAction, Parser};
use getset::{Setters, WithSetters};

/// Galaxy Flow Command Line Interface
///
/// GxlCmd是Galaxy Flow的命令行接口结构体，用于解析和处理命令行参数。
///
/// Galaxy Flow Command Line Interface
///
/// GxlCmd is the command line interface structure for Galaxy Flow, used to parse and process command line arguments.
#[derive(Parser, Debug, Clone, WithSetters, Setters)] // requires `derive` feature
#[command(version, about = "Galaxy Flow - A powerful workflow automation tool", long_about = None)]
#[command(
    after_help = "Examples:\n  gxl -e dev -f ./config.gxl flow1 flow2\n  gxl -e prod --cmd-arg \"-x -y\" flow1\n  gxl -e test --dryrun flow1\n\n示例：\n  gxl -e dev -f ./config.gxl flow1 flow2\n  gxl -e prod --cmd-arg \"-x -y\" flow1\n  gxl -e test --dryrun flow1"
)]
#[getset(set_with = "pub")]
pub struct GxlCmd {
    /// 环境名称 / Environment name
    ///
    /// 指定运行环境，例如：dev, test, prod
    ///
    /// Specify the runtime environment, e.g.: dev, test, prod
    /// 示例/Example: -e dev
    #[arg(short = 'e', long = "env", default_value = "default")]
    #[getset(set = "pub")]
    env: String,

    /// 位置参数流程名称 / Positional flow names
    ///
    /// 作为位置参数指定的流程名称列表，会与flows参数合并
    ///
    /// List of flow names specified as positional arguments, will be merged with the flows parameter
    /// 示例/Example: gxl flow1 flow2
    #[arg()] // Positional arguments
    pub flows: Vec<String>,

    /// 调试级别 / Debug level
    ///
    /// 设置调试输出级别，数值越大输出越详细
    ///
    /// Set the debug output level, higher values produce more detailed output
    /// 示例/Example: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,

    /// 配置文件路径 / Configuration file path
    ///
    /// 指定GXL配置文件的路径
    /// 默认工作配置: ./_gal/work.gxl
    /// 默认管理配置: ./_gal/adm.gxl
    ///
    /// Specify the path to the GXL configuration file
    /// Default work config: ./_gal/work.gxl
    /// Default admin config: ./_gal/adm.gxl
    /// 示例/Example: -c ./config.gxl
    #[arg(short = 'c', long = "conf")]
    pub conf: Option<String>,

    /// 日志配置 / Log configuration
    ///
    /// 配置日志输出级别和格式
    ///
    /// Configure log output level and format
    /// 示例/Example: --log cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,

    /// 静默模式 / Quiet mode
    ///
    /// 启用静默模式，减少输出信息
    ///
    /// Enable quiet mode to reduce output information
    /// 示例/Example: -q
    #[arg(short = 'q', long = "quiet", action = ArgAction::SetTrue,default_value = "false" )]
    pub quiet: bool,

    /// 命令参数 / Command arguments
    ///
    /// 传递给流程的命令行参数，允许以连字符开头的值
    ///
    /// Command line arguments passed to the flow, allows values starting with hyphens
    /// 示例/Example: --cmd-arg "-x -y"
    #[arg(
        long = "cmd-arg",
        value_name = "ARG",
        num_args = 1,
        allow_hyphen_values = true
    )]
    pub cmd_args: Vec<String>,

    /// 试运行模式 / Dry run mode
    ///
    /// 启用试运行模式，只显示将要执行的操作而不实际执行
    ///
    /// Enable dry run mode, only show what operations would be performed without actually executing them
    /// 示例/Example: --dryrun
    #[arg(long = "dryrun", action = ArgAction::SetTrue, default_value = "false")]
    pub dryrun: bool,

    /// AI辅助模式 / AI assistance mode
    ///
    /// 启用AI辅助功能，在出错时提供智能诊断和建议
    ///
    /// Enable AI assistance features, provide intelligent diagnosis and suggestions when errors occur
    /// 示例/Example: --ai
    #[arg(long = "ai", action = ArgAction::SetTrue, default_value = "false")]
    pub ai: bool,

    /// 模块更新 / Module update
    ///
    /// 更新远程GXL模块
    ///
    /// Update remote GXL modules
    /// 示例/Example: --mod-up
    #[arg(long = "mod_up", action = ArgAction::SetTrue, default_value = "false")]
    pub mod_update: bool,
}

impl GxlCmd {
    /// 获取所有流程名称
    ///
    /// 合并flows和flow_names字段，返回所有要执行的流程名称列表
    /// 优先使用flows参数，如果为空则使用flow_names位置参数
    /// 注意：flow_names字段已弃用，建议使用flows字段
    ///
    /// Get all flow names
    ///
    /// Merge the flows and flow_names fields, return a list of all flow names to be executed
    /// Priority is given to the flows parameter, if empty then use flow_names positional arguments
    /// Note: The flow_names field is deprecated, it is recommended to use the flows field
    pub fn get_all_flows(&self) -> Vec<String> {
        if !self.flows.is_empty() {
            self.flows.clone()
        } else if !self.flows.is_empty() {
            // 输出弃用警告 / Output deprecation warning
            eprintln!("Warning: flow_names field is deprecated, please use flows field instead");
            self.flows.clone()
        } else {
            Vec::new()
        }
    }

    /// 将环境字符串按逗号分解为向量
    ///
    /// 将env字段中的字符串按逗号分隔，返回字符串向量
    /// 如果env为空字符串，则返回空向量
    ///
    /// Split environment string by comma into vector
    ///
    /// Split the string in the env field by comma and return a vector of strings
    /// If env is an empty string, return an empty vector
    pub fn get_env_list(&self) -> Vec<String> {
        if self.env.is_empty() {
            Vec::new()
        } else {
            self.env
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
    }

    /// 验证命令行参数的有效性
    ///
    /// 检查命令行参数是否满足基本要求，包括：
    /// 1. 必须指定配置文件
    /// 2. 如果指定了cmd_args，必须同时指定flows或flow_names
    ///
    /// Validate the effectiveness of command line arguments
    ///
    /// Check if the command line arguments meet the basic requirements, including:
    /// 1. Configuration file must be specified
    /// 2. If cmd_args is specified, flows or flow_names must also be specified
    pub fn validate(&self) -> Result<(), String> {
        if self.conf.is_none() {
            return Err("Configuration file is required".to_string());
        }
        if self.get_all_flows().is_empty() && !self.cmd_args.is_empty() {
            return Err("Cannot specify cmd_args without flows".to_string());
        }
        Ok(())
    }
}

impl Default for GxlCmd {
    fn default() -> Self {
        Self {
            env: "default".to_string(),
            flows: Vec::new(),
            debug: 0,
            conf: None,
            log: None,
            quiet: false,
            cmd_args: Vec::new(),
            dryrun: false,
            ai: false,
            mod_update: false,
        }
    }
}
