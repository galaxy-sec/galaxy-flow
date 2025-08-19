use home::home_dir;
use orion_common::serde::Yamlable;
use orion_error::{ErrorOwe, ToStructError, UvsResFrom};
use orion_variate::addr::access_ctrl::{serv::NetAccessCtrl, Rule, Unit};

use crate::err::{RunReason, RunResult};

pub struct Galaxy {}
const NET_ACCESS_CTRL_FILE: &str = "net_accessor_ctrl.yml";
impl Galaxy {
    /// 初始化 Galaxy 环境
    ///
    /// 包括：
    /// - 创建 `${HOME}/.galaxy` 目录
    /// - 基于 RedirectService demo 数据创建序列化的 redirect.yml 文件
    pub fn env_init() -> RunResult<()> {
        // 获取家目录并构建环境目录
        let galaxy_dir = home_dir()
            .ok_or_else(|| RunReason::from_res("Cannot find home directory".into()).to_err())?
            .join(".galaxy");

        // 创建目录
        if !galaxy_dir.exists() {
            std::fs::create_dir_all(&galaxy_dir).owe_res()?;
        }

        // 构建正确的 RedirectService demo 数据

        let redirect_path = galaxy_dir.join(NET_ACCESS_CTRL_FILE);
        if redirect_path.exists() {
            println!(
                " {} exists! , redirect init ignore",
                redirect_path.display()
            );
            return Ok(());
        }

        // 创建演示重定向规则
        let rules = vec![Rule::new("https://google.com/*", "https://google.cn/")];

        // 创建重定向单元
        let unit = Unit::new(rules, None, None);

        // 创建服务实例
        let service = NetAccessCtrl::new(vec![unit], true);
        service.save_yml(&redirect_path).owe_res()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssertWithMsg;

    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn test_env_init_success() {
        // 使用临时目录进行测试
        let temp_dir = PathBuf::from("./temp");
        let galaxy_dir = temp_dir.join(".galaxy");

        // 临时修改HOME环境变量
        let old_home = std::env::var("HOME").unwrap();
        std::env::set_var("HOME", temp_dir);

        // 确保清理
        let _cleanup = || {
            std::env::set_var("HOME", &old_home);
        };

        // 执行初始化
        let result = Galaxy::env_init();
        assert!(result.is_ok(), "Environment init should succeed");
        let conf_path = galaxy_dir.join(NET_ACCESS_CTRL_FILE);

        // 验证目录和文件创建
        assert!(galaxy_dir.exists());
        assert!(conf_path.exists());

        NetAccessCtrl::from_yml(&conf_path).assert("redict");
        // 验证文件内容包含关键字段
        let content = fs::read_to_string(conf_path).unwrap();
        println!("{content}");
        assert!(content.contains("units:"));
        assert!(content.contains("enable: true"));
    }
}
