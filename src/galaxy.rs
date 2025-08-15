use home::home_dir;
use orion_ai::{AiConfig, RoleConfigManager};
use orion_common::serde::Yamlable;
use orion_error::{ErrorOwe, UvsResFrom};
use orion_variate::addr::access_ctrl::{serv::NetAccessCtrl, Rule, Unit};

use crate::{
    const_val::gxl_const::{AI_CONF_FILE, AI_ROLE_FILE, NET_ACCESS_CTRL_FILE},
    err::{RunReason, RunResult},
};

pub struct Galaxy {}

impl Galaxy {
    /// 初始化 Galaxy 环境
    ///
    /// 包括：
    /// - 创建 `${HOME}/.galaxy` 目录
    /// - 基于 RedirectService demo 数据创建序列化的 redirect.yml 文件
    pub fn env_init() -> RunResult<()> {
        // 获取家目录并构建环境目录
        let galaxy_dir = home_dir()
            .ok_or_else(|| RunReason::from_res("Cannot find home directory".into()))?
            .join(".galaxy");

        // 创建目录
        if !galaxy_dir.exists() {
            std::fs::create_dir_all(&galaxy_dir).owe_res()?;
        }

        // 构建正确的 RedirectService demo 数据

        let net_ctrl_path = galaxy_dir.join(NET_ACCESS_CTRL_FILE);
        if net_ctrl_path.exists() {
            println!(
                " {} exists! , net access ctrl init ignore",
                net_ctrl_path.display()
            );
        } else {
            let rules = vec![Rule::new("https://google.com/*", "https://google.cn/")];
            let unit = Unit::new(rules, None, None);
            let service = NetAccessCtrl::new(vec![unit], true);
            service.save_yml(&net_ctrl_path).owe_res()?;
        }

        let ai_conf_path = galaxy_dir.join(AI_CONF_FILE);
        if ai_conf_path.exists() {
            println!(
                " {} exists! , ai provider init ignore",
                ai_conf_path.display()
            );
        } else {
            AiConfig::example().save_yml(&ai_conf_path).owe_res()?;
        }
        let ai_role_path = galaxy_dir.join(AI_ROLE_FILE);
        if !ai_role_path.exists() {
            RoleConfigManager::default()
                .save_yml(&ai_role_path)
                .owe_res()?;
        } else {
            println!(" {} exists! , ai role init ignore", ai_role_path.display());
        }
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
