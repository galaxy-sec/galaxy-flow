use std::{collections::HashMap, path::PathBuf};

use derive_more::From;
use dirs::home_dir;

use crate::{var::VarDict, ExecResult};

use super::global::{load_secfile, setup_gxlrun_vars, setup_start_vars};

#[derive(Debug, Clone, Default, PartialEq, From, Getters)]
pub struct VarSpace {
    inherited: VarDict,
    global: VarDict,
    nameds: HashMap<String, VarDict>,
}

pub fn sec_value_default_path() -> PathBuf {
    galaxy_dot_path().join("sec_value.yml")
}
pub fn galaxy_dot_path() -> PathBuf {
    home_dir()
        .map(|x| x.join(".galaxy"))
        .unwrap_or(PathBuf::from("./"))
}

impl VarSpace {
    pub fn sys_init() -> ExecResult<VarSpace> {
        let mut var_space = VarSpace::default();
        load_secfile(&mut var_space.inherited)?;
        setup_start_vars(&mut var_space.inherited)?;
        setup_gxlrun_vars(&mut var_space.inherited)?;
        var_space.global = var_space.inherited.clone();
        Ok(var_space)
    }
    pub fn inherit_init(mut origin: VarSpace, isolate: bool) -> ExecResult<VarSpace> {
        if isolate {
            let mut ins = Self {
                inherited: origin.inherited.clone(),
                global: origin.inherited,
                ..Default::default()
            };
            setup_gxlrun_vars(&mut ins.inherited)?;
            Ok(ins)
        } else {
            setup_gxlrun_vars(&mut origin.inherited)?;
            setup_gxlrun_vars(&mut origin.global)?;
            Ok(origin)
        }
    }
    pub fn global_mut(&mut self) -> &mut VarDict {
        &mut self.global
    }

    pub fn nameds_mut(&mut self) -> &mut HashMap<String, VarDict> {
        &mut self.nameds
    }
}
#[derive(Debug, Clone, Default, PartialEq, From)]
pub enum DictUse {
    #[default]
    Global,
    Named(String),
}

#[cfg(test)]
mod tests {
    use crate::{
        execution::{dict::sec_value_default_path, global::load_secfile},
        traits::Getter,
        var::VarDict,
    };

    use orion_error::TestAssertWithMsg;
    use std::{env::temp_dir, fs::File, io::Write};

    #[test]
    fn test_load_secfile_with_values() {
        // 创建临时目录和文件
        //let dir = PathBuf::from("./temp");
        let dir = temp_dir();
        let file_path = dir.join("sec_value.yml");
        if file_path.exists() {
            std::fs::remove_file(&file_path).assert("remove file");
        }

        // 写入测试内容
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "key1: value1\nkey2: value2").unwrap();

        // 创建 VarSpace 实例并加载文件
        let mut var_dict = VarDict::new("test");

        // 临时修改路径指向我们的测试文件
        let original_path = sec_value_default_path();
        std::env::set_var("GAL_SEC_FILE_PATH", file_path.to_str().unwrap());

        let _ = load_secfile(&mut var_dict).assert("load secfile");

        // 验证全局变量
        assert!(var_dict.contains_key("SEC_KEY1"));
        assert!(var_dict.contains_key("SEC_KEY2"));
        assert_eq!(
            format!("{}", var_dict.get("SEC_KEY1").unwrap()),
            "******".to_string()
        );
        // 清理
        std::env::set_var("GAL_SEC_FILE_PATH", original_path);
    }
}
