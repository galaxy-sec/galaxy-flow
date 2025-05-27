use std::{collections::HashMap, env::home_dir, path::PathBuf};

use derive_more::From;
use orion_error::ErrorConv;
use orion_syspec::{
    types::Configable,
    vars::{ValueDict, ValueType},
};

use crate::{
    err::RunResult,
    traits::Setter,
    var::{SecVar, VarDict},
};

#[derive(Debug, Clone, Default, PartialEq, From, Getters)]
pub struct VarSpace {
    globle: VarDict,
    nameds: HashMap<String, VarDict>,
}

pub fn sec_value_default_path() -> PathBuf {
    home_dir()
        .map(|x| x.join(".galaxy/sec_value.yml"))
        .unwrap_or(PathBuf::from("./"))
}
impl VarSpace {
    pub fn globle_mut(&mut self) -> &mut VarDict {
        &mut self.globle
    }

    pub fn nameds_mut(&mut self) -> &mut HashMap<String, VarDict> {
        &mut self.nameds
    }

    pub(crate) fn load_secfile(&mut self) -> RunResult<()> {
        let env_path = std::env::var("GAL_SEC_FILE_PATH").map(PathBuf::from);
        let default = sec_value_default_path();
        let path = env_path.unwrap_or(default);
        if path.exists() {
            let dict = ValueDict::from_conf(&path).err_conv()?;
            info!(target: "exec","  load {}", path.display());
            for (k, v) in dict.iter() {
                self.globle.set(
                    format!("SEC_{}", k.to_uppercase()),
                    SecVar::sec_value(v.to_string()),
                );
            }
        } else {
            let mut default = ValueDict::new();
            default.insert("example_key1", ValueType::from("value"));
            default.save_conf(&path).err_conv()?;
        }
        Ok(())
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
    use crate::{execution::dict::sec_value_default_path, traits::Getter};

    use super::VarSpace;
    use std::{fs::File, io::Write};
    use tempfile::tempdir;

    #[test]
    fn test_load_secfile_with_values() {
        // 创建临时目录和文件
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("sec_value.yml");

        // 写入测试内容
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "key1: value1\nkey2: value2").unwrap();

        // 创建 VarSpace 实例并加载文件
        let mut var_space = VarSpace::default();

        // 临时修改路径指向我们的测试文件
        let original_path = sec_value_default_path();
        std::env::set_var("GAL_SEC_FILE_PATH", file_path.to_str().unwrap());

        let result = var_space.load_secfile();
        assert!(result.is_ok());

        // 验证全局变量
        let globle = var_space.globle();
        assert!(globle.contains_key("SEC_KEY1"));
        assert!(globle.contains_key("SEC_KEY2"));
        assert_eq!(
            format!("{}", globle.get("SEC_KEY1").unwrap()),
            "******".to_string()
        );
        // 清理
        std::env::set_var("GAL_SEC_FILE_PATH", original_path);
    }
}
